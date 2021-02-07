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
            output_block(w, inputs, 0)?;
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

    let indent_str: String = "  ".repeat(indent_count);
    for field in &inputs.fields {
        // TODO each value should output their value as JSON encoded value
        writeln!(w, "{}{}: \"{}\"", indent_str, field.0, field.1)?;
    }

    for (&name, input) in inputs.inputs.iter().chain(&inputs.stacks) {
        if name == "next" {
            output_block(w, &input, indent_count)?;
        } else {
            write!(w, "{}{}: ", indent_str, name)?;
            output_block(w, &input, indent_count + 1)?;
        }
    }

    Ok(())
}
