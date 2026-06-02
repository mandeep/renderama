use clap::ValueEnum;
use rand_pcg::Pcg64Mcg;
use strum::{Display, IntoStaticStr};

use scene::Scene;

mod cornell_box_boxes;
mod cornell_box_bunny;
mod cornell_box_dragon;
mod cornell_box_objects;
mod cornell_box_uv;
mod energy_conservation;
mod hyperion;
mod random_spheres;
mod spheres_in_box;
mod three_spheres;
mod veach_mis;
mod white_furnace;

pub use self::cornell_box_boxes::*;
pub use self::cornell_box_bunny::*;
pub use self::cornell_box_dragon::*;
pub use self::cornell_box_objects::*;
pub use self::cornell_box_uv::*;
pub use self::energy_conservation::*;
pub use self::hyperion::*;
pub use self::random_spheres::*;
pub use self::spheres_in_box::*;
pub use self::three_spheres::*;
pub use self::veach_mis::*;
pub use self::white_furnace::*;


#[derive(Clone, Debug, Display, IntoStaticStr, ValueEnum)]
#[value(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Scenes {
    CornellBoxBoxes,
    CornellBoxBunny,
    CornellBoxDragon,
    CornellBoxObjects,
    CornellBoxUV,
    EnergyConservation,
    Hyperion,
    RandomSpheres,
    SpheresInBox,
    ThreeSpheres,
    VeachMis,
    WhiteFurnace,
}

impl Scenes {
    pub fn load(&self, width: Option<usize>, height: Option<usize>, rng: &mut Pcg64Mcg) -> Scene {
        match self {
            Scenes::CornellBoxBoxes => cornell_box_scene(width, height),
            Scenes::CornellBoxBunny => cornell_box_bunny_scene(width, height),
            Scenes::CornellBoxDragon => cornell_box_dragon_scene(width, height),
            Scenes::CornellBoxObjects => cornell_box_object_scene(width, height),
            Scenes::CornellBoxUV => cornell_box_uv_scene(width, height),
            Scenes::EnergyConservation => energy_conservation_scene(width, height),
            Scenes::Hyperion => hyperion_scene(width, height),
            Scenes::RandomSpheres => random_spheres_scene(width, height, rng),
            Scenes::SpheresInBox => spheres_in_box_scene(width, height, rng),
            Scenes::ThreeSpheres => three_spheres_scene(width, height),
            Scenes::VeachMis => veach_mis_scene(width, height),
            Scenes::WhiteFurnace => white_furnace_scene(width, height),
        }
    }
}