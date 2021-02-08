use super::*;
use crate::broadcaster::Broadcaster;
use crate::file::ScratchFile;
use crate::runtime::Global;
use crate::sprite::{Sprite, SpriteID};
use crate::thread::BlockInputs;
use colored::Colorize;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

pub async fn fileviewer(file_path: &Path) -> Result<()> {
    let scratch_file = ScratchFile::parse(BufReader::new(File::open(file_path)?))?;
    let block_inputs = block_inputs(&scratch_file.project.targets).await?;

    let mut w = BufWriter::new(std::io::stdout());
    output_block_inputs(&mut w, &block_inputs)?;
    writeln!(w, "{}", "ScratchFile structure".bold())?;
    writeln!(w, "{:#?}", scratch_file)?;
    w.flush()?;
    Ok(())
}

#[derive(Debug)]
struct SpriteBlocks {
    name: String,
    id: SpriteID,
    block_inputs: Vec<BlockInputs>,
}

async fn block_inputs(targets: &[file::Target]) -> Result<Vec<SpriteBlocks>> {
    let global = Arc::new(Global::new(
        &HashMap::new(),
        &Vec::new(),
        Broadcaster::new(),
    ));

    let mut block_inputs: Vec<SpriteBlocks> = Vec::with_capacity(targets.len());

    for target in targets {
        let (id, sprite) = Sprite::new(global.clone(), target.clone(), false).await?;
        block_inputs.push(SpriteBlocks {
            name: target.name.clone(),
            id,
            block_inputs: sprite.block_inputs().await,
        });
    }
    Ok(block_inputs)
}

fn output_block_inputs<W>(w: &mut W, sprites: &[SpriteBlocks]) -> Result<()>
where
    W: std::io::Write,
{
    for sprite in sprites {
        let sprite_id_truncated: String = format!("{}", sprite.id).chars().take(8).collect();
        writeln!(
            w,
            "{}",
            format!("Sprite {} ({})", sprite_id_truncated, sprite.name).bold()
        )?;

        for (thread_id, inputs) in sprite.block_inputs.iter().enumerate() {
            writeln!(w, "{}", format!("Thread {}", thread_id).underline())?;
            output_block(w, inputs, 1)?;
        }

        writeln!(w)?;
    }
    Ok(())
}

fn output_block<W>(w: &mut W, inputs: &BlockInputs, indent_count: usize) -> Result<()>
where
    W: std::io::Write,
{
    writeln!(w, "{} {}", inputs.info.name, inputs.info.id)?;

    const SINGLE_INDENT: &str = "|   ";
    let base: String = SINGLE_INDENT.repeat(indent_count);
    let raised: String = base.clone() + "| ";
    let base_plus_indent: String = base.clone() + SINGLE_INDENT;
    for (&id, value) in &inputs.fields {
        writeln!(w, "{}{}: {}", raised, id, value)?;
    }

    for (&name, input) in inputs.inputs.iter().chain(&inputs.stacks) {
        if name == "next" {
            write!(w, "{}", base)?;
            output_block(w, &input, indent_count)?;
        } else {
            writeln!(w, "{}{}:", raised, name)?;
            write!(w, "{}", base_plus_indent)?;
            output_block(w, &input, indent_count + 1)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::file::ScratchFile;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_block_inputs() {
        {
            assert!(block_inputs(&Vec::new()).await.unwrap().is_empty());
        }
        {
            let dir = std::path::Path::new(file!())
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("test_saves")
                .join("say.sb3");
            let file = std::fs::File::open(dir).unwrap();
            let scratch_file = ScratchFile::parse(&file).unwrap();
            assert!(!block_inputs(&scratch_file.project.targets)
                .await
                .unwrap()
                .is_empty());
        }
    }

    #[tokio::test]
    async fn test_output_block_inputs() {
        {
            let mut result: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            output_block_inputs(&mut result, &Vec::new()).unwrap();
            assert!(result.get_ref().is_empty());
        }
        {
            let dir = std::path::Path::new(file!())
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("test_saves")
                .join("say.sb3");
            let file = std::fs::File::open(dir).unwrap();
            let scratch_file = ScratchFile::parse(&file).unwrap();
            let block_inputs = block_inputs(&scratch_file.project.targets).await.unwrap();

            let mut result: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            output_block_inputs(&mut result, &block_inputs).unwrap();
            assert!(!result.get_ref().is_empty());
        }
    }
}
