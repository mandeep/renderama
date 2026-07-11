use std::collections::HashMap;
use std::convert::Infallible;

use rand::TryRng;

use crate::lights::Light;
use crate::materials::{Material, MaterialId};
use crate::texture::{Texture, TextureId};


/// Add a push_into method on Vec so that we can avoid calling .into() on every push
pub trait PushInto<T> {
    fn push_into(&mut self, item: impl Into<T>);
}

impl<T> PushInto<T> for Vec<T> {
    fn push_into(&mut self, item: impl Into<T>) {
        self.push(item.into());
    }
}

/// Add an insert_into on HashMap so that we can avoid calling .into() on every insertion
pub trait InsertInto<K, V> {
    fn insert_into(&mut self, key: impl Into<K>, value: impl Into<V>);
}

impl<K: Eq + std::hash::Hash, V> InsertInto<K, V> for HashMap<K, V> {
    fn insert_into(&mut self, key: impl Into<K>, value: impl Into<V>) {
        self.insert(key.into(), value.into());
    }
}

pub trait AddMaterial {
    fn add_material(&mut self, material: impl Into<Material>) -> MaterialId;
}

impl AddMaterial for Vec<Material> {
    fn add_material(&mut self, material: impl Into<Material>) -> MaterialId {
        let id = MaterialId(self.len() as u32);
        self.push(material.into());
        id
    }
}

pub trait AddTexture {
    fn add_texture(&mut self, texture: impl Into<Texture>) -> TextureId;
}

impl AddTexture for Vec<Texture> {
    fn add_texture(&mut self, texture: impl Into<Texture>) -> TextureId {
        let id = TextureId(self.len() as u32);
        self.push(texture.into());
        id
    }
}

pub trait AddLight {
    fn add_light(&mut self, light: impl Into<Light>);
}

impl AddLight for Vec<Light> {
    fn add_light(&mut self, light: impl Into<Light>) {
        self.push(light.into());
    }
}

pub struct DummyRng;

impl TryRng for DummyRng {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok(0)
    }

    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        Ok(0)
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        dst.fill(0);
        Ok(())
    }
}