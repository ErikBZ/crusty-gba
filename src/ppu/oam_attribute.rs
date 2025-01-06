pub struct OamAttribute {
    y_coord: u32,
    x_coord: u32,
    is_rot_scale: bool,
    double_size_or_disable: bool,
    obj_mode: u32,
    obj_mosaic: bool,
    colos: bool,
    obj_shape: u32,
    obj_size: u32,
    character_name: u32,
    priority: u32,
    palette: u32,
}

impl From<&[u32; 3]> for OamAttribute {
    fn from(value: &[u32; 3]) -> Self {
        todo!() 
    }
}
