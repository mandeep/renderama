#![allow(unused_imports)]
use scene::Scene;

mod cornell_box_boxes;
mod cornell_box_bunny;
mod cornell_box_dragon;
mod cornell_box_objects;
mod random_spheres;
mod spheres_in_box;
mod three_spheres;
mod veach_mis;

pub use self::cornell_box_boxes::*;
pub use self::cornell_box_bunny::*;
pub use self::cornell_box_dragon::*;
pub use self::cornell_box_objects::*;
pub use self::random_spheres::*;
pub use self::spheres_in_box::*;
pub use self::three_spheres::*;
pub use self::veach_mis::*;


#[derive(Debug, Clone)]
pub enum Scenario {
    CornellBoxBoxes,
    CornellBoxBunny,
    CornellBoxDragon,
    CornellBoxObjects,
    RandomSpheres,
    SpheresInBox,
    ThreeSpheres,
    VeachMis,
}

impl Scenario {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cornell_box_boxes" => Some(Scenario::CornellBoxBoxes),
            "cornell_box_bunny" => Some(Scenario::CornellBoxBunny),
            "cornell_box_dragon" => Some(Scenario::CornellBoxDragon),
            "cornell_box_objects" => Some(Scenario::CornellBoxObjects),
            "random_spheres" => Some(Scenario::RandomSpheres),
            "spheres_in_box" => Some(Scenario::SpheresInBox),
            "three_spheres" => Some(Scenario::ThreeSpheres),
            "veach_mis" => Some(Scenario::VeachMis),
            _ => None,
        }
    }

    pub fn load(&self, width: Option<usize>, height: Option<usize>) -> Scene {
        match self {
            Scenario::CornellBoxBoxes => cornell_box_scene(width, height),
            Scenario::CornellBoxBunny => cornell_box_bunny_scene(width, height),
            Scenario::CornellBoxDragon => cornell_box_dragon_scene(width, height),
            Scenario::CornellBoxObjects => cornell_box_object_scene(width, height),
            Scenario::RandomSpheres => random_spheres_scene(width, height),
            Scenario::SpheresInBox => spheres_in_box_scene(width, height),
            Scenario::ThreeSpheres => three_spheres_scene(width, height),
            Scenario::VeachMis => veach_mis_scene(width, height),
        }
    }
}