use super::*;
use palette::{Hsv, IntoColor};
use palette::{Mix, Srgb};

pub fn get_block(name: &str, id: BlockID, runtime: Runtime) -> Result<Box<dyn Block>> {
    Ok(match name {
        "penDown" => Box::new(PenDown::new(id, runtime)),
        "penUp" => Box::new(PenUp::new(id, runtime)),
        "setPenColorToColor" => Box::new(SetPenColorToColor::new(id, runtime)),
        "setPenSizeTo" => Box::new(SetPenSizeTo::new(id, runtime)),
        "clear" => Box::new(Clear::new(id, runtime)),
        "setPenShadeToNumber" => Box::new(SetPenShadeToNumber::new(id, runtime)),
        "setPenHueToNumber" => Box::new(SetPenHueToNumber::new(id, runtime)),
        _ => return Err(Error::msg(format!("{} does not exist", name))),
    })
}

#[derive(Debug)]
pub struct PenDown {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl PenDown {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait]
impl Block for PenDown {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "PenDown",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let mut runtime = self.runtime.sprite.write().await;
        let center = runtime.rectangle().center;
        runtime.pen().pen_down(&center);
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct PenUp {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl PenUp {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait]
impl Block for PenUp {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "PenUp",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        self.runtime.sprite.write().await.pen().pen_up();
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SetPenColorToColor {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    color: Box<dyn Block>,
}

impl SetPenColorToColor {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            color: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for SetPenColorToColor {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetPenColorToColor",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("COLOR", self.color.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "COLOR" {
            self.color = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let color: Srgb<u8> = self.color.value().await?.try_into()?;
        self.runtime.sprite.write().await.pen().set_color(color);
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SetPenSizeTo {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    size: Box<dyn Block>,
}

impl SetPenSizeTo {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            size: Box::new(EmptyInput {}),
        }
    }
}

#[async_trait]
impl Block for SetPenSizeTo {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetPenSizeTo",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("SIZE", self.size.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "SIZE" {
            self.size = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let size: f64 = self.size.value().await?.try_into()?;
        self.runtime.sprite.write().await.pen().set_size(size);
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct Clear {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
}

impl Clear {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
        }
    }
}

#[async_trait]
impl Block for Clear {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "Clear",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![],
            vec![("next", &self.next)],
        )
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        self.runtime.sprite.write().await.pen().clear();
        Next::continue_(self.next)
    }
}

#[derive(Debug)]
pub struct SetPenShadeToNumber {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    shade: Box<dyn Block>,
}

impl SetPenShadeToNumber {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            shade: Box::new(EmptyInput {}),
        }
    }

    fn set_shade(color: &Hsv, shade: f32) -> Hsv {
        // https://github.com/LLK/scratch-vm/blob/c6962cb390ba2835d64eb21c0456707b51642084/src/extensions/scratch3_pen/index.js#L718
        let mut new_shade = shade % 200.0;
        if new_shade < 0.0 {
            new_shade += 200.0
        }

        // https://github.com/LLK/scratch-vm/blob/c6962cb390ba2835d64eb21c0456707b51642084/src/extensions/scratch3_pen/index.js#L750
        let constrained_shade = if new_shade > 100.0 {
            200.0 - new_shade
        } else {
            new_shade
        };

        let bright = Hsv::new(color.hue, 1.0, 1.0);
        if constrained_shade < 50.0 {
            Hsv::new(0.0, 0.0, 0.0).mix(&bright, (10.0 + shade) / 60.0)
        } else {
            bright.mix(&Hsv::new(0.0, 0.0, 1.0), (shade - 50.0) / 60.0)
        }
    }
}

#[async_trait]
impl Block for SetPenShadeToNumber {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetPenShadeToNumber",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("SHADE", self.shade.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "SHADE" {
            self.shade = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let shade: f64 = self.shade.value().await?.try_into()?;
        let mut runtime = self.runtime.sprite.write().await;
        let color = runtime.pen().color();
        let hsv = rgb_to_hsv(color);
        let new_color = SetPenShadeToNumber::set_shade(&hsv, shade as f32);
        runtime.pen().set_color(hsv_to_rgb(&new_color));
        Next::continue_(self.next)
    }
}

fn rgb_to_hsv(rgb: &Srgb<u8>) -> Hsv {
    Srgb::<f32>::new(
        rgb.red as f32 / 255.0,
        rgb.green as f32 / 255.0,
        rgb.blue as f32 / 255.0,
    )
    .into_hsv()
}

fn hsv_to_rgb(hsv: &Hsv) -> Srgb<u8> {
    let rgb_float: Srgb = hsv.into_rgb().into_encoding();
    Srgb::new(
        (rgb_float.red * 255.0) as u8,
        (rgb_float.green * 255.0) as u8,
        (rgb_float.blue * 255.0) as u8,
    )
}

#[derive(Debug)]
pub struct SetPenHueToNumber {
    id: BlockID,
    runtime: Runtime,
    next: Option<BlockID>,
    hue: Box<dyn Block>, // [0, 100]
}

impl SetPenHueToNumber {
    pub fn new(id: BlockID, runtime: Runtime) -> Self {
        Self {
            id,
            runtime,
            next: None,
            hue: Box::new(EmptyInput {}),
        }
    }

    fn set_hue(color: &Hsv, hue: f32) -> Hsv {
        #[allow(clippy::float_cmp)]
        if hue == 200.0 {
            Hsv::new(360.0, 0.0, 0.0)
        } else {
            Hsv::new(hue / 200.0 * 360.0, color.saturation, color.value)
        }
    }
}

#[async_trait]
impl Block for SetPenHueToNumber {
    fn block_info(&self) -> BlockInfo {
        BlockInfo {
            name: "SetPenHueToNumber",
            id: self.id,
        }
    }

    fn block_inputs(&self) -> BlockInputsPartial {
        BlockInputsPartial::new(
            self.block_info(),
            vec![],
            vec![("HUE", self.hue.as_ref())],
            vec![("next", &self.next)],
        )
    }

    fn set_input(&mut self, key: &str, block: Box<dyn Block>) {
        if key == "HUE" {
            self.hue = block;
        }
    }

    fn set_substack(&mut self, key: &str, block: BlockID) {
        if key == "next" {
            self.next = Some(block);
        }
    }

    async fn execute(&mut self) -> Result<Next> {
        let hue: f64 = self.hue.value().await?.try_into()?;
        let mut runtime = self.runtime.sprite.write().await;
        let new_color = SetPenHueToNumber::set_hue(&rgb_to_hsv(&runtime.pen().color()), hue as f32);
        runtime.pen().set_color(hsv_to_rgb(&new_color));
        Next::continue_(self.next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    #[case(Hsv::new(0.0, 0.0, 0.0), 0.0, Hsv::new(0.0, 0.16666667, 0.16666667))]
    #[case(Hsv::new(0.0, 0.0, 1.0), 0.0, Hsv::new(0.0, 0.16666667, 0.16666667))]
    #[case(Hsv::new(0.0, 0.0, 0.0), 100.0, Hsv::new(0.0, 0.16666669, 1.0))]
    #[case(Hsv::new(0.0, 0.0, 1.0), 100.0, Hsv::new(0.0, 0.16666669, 1.0))]
    #[case(Hsv::new(0.0, 0.0, 0.0), 50.0, Hsv::new(0.0, 1.0, 1.0))]
    #[case(Hsv::new(240.0, 1.0, 1.0), 50.0, Hsv::new(240.0, 1.0, 1.0))]
    fn test_set_shade(#[case] color: Hsv, #[case] shade: f32, #[case] expected: Hsv) {
        assert_eq!(SetPenShadeToNumber::set_shade(&color, shade), expected);
    }

    #[rstest]
    #[case(Hsv::new(0.0, 0.0, 0.0), 0.0, Hsv::new(0.0, 0.0, 0.0))]
    #[case(Hsv::new(0.0, 1.0, 1.0), 0.0, Hsv::new(0.0, 1.0, 1.0))]
    #[case(Hsv::new(0.0, 0.0, 0.0), 50.0, Hsv::new(90.0, 0.0, 0.0))]
    #[case(Hsv::new(0.0, 0.0, 0.0), 100.0, Hsv::new(180.0, 0.0, 0.0))]
    #[case(Hsv::new(0.0, 0.0, 0.0), 200.0, Hsv::new(360.0, 0.0, 0.0))]
    fn test_set_hue(#[case] color: Hsv, #[case] hue: f32, #[case] expected: Hsv) {
        assert_eq!(SetPenHueToNumber::set_hue(&color, hue), expected);
    }
}
