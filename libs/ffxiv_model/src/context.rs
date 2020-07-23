use alloc::{format, sync::Arc};

use enum_iterator::IntoEnumIterator;
use futures::{
    stream::{FuturesUnordered, TryStreamExt},
    FutureExt,
};
use hashbrown::HashMap;

use ffxiv_parser::Eqdp;
use renderer::{Renderer, Texture, TextureFormat};
use sqpack_reader::{Package, Result};

use crate::constants::{BodyId, ModelPart};
use crate::shader_holder::ShaderHolder;
use crate::texture_cache::TextureCache;

pub struct Context {
    pub(crate) shader_holder: ShaderHolder,
    pub(crate) texture_cache: TextureCache,
    pub(crate) empty_texture: Arc<Texture>,
    equipment_deformer_parameters: HashMap<BodyId, Eqdp>,
}

impl Context {
    pub async fn new(renderer: &Renderer, package: &dyn Package) -> Result<Self> {
        let empty_texture = Self::create_empty_texture(renderer).await;
        let equipment_deformer_parameters = Self::create_equipment_deformer_parameters(package).await?;

        Ok(Self {
            shader_holder: ShaderHolder::new(renderer),
            texture_cache: TextureCache::new(),
            empty_texture,
            equipment_deformer_parameters,
        })
    }

    #[allow(dead_code)]
    pub fn get_deformed_body_id(&self, body_id: BodyId, model_id: u16, model_part: ModelPart) -> BodyId {
        if body_id == BodyId::MidlanderMale {
            return BodyId::MidlanderMale;
        }

        let eqdp = self.equipment_deformer_parameters.get(&body_id).unwrap();
        if eqdp.has_model(model_id, model_part as u8) {
            body_id
        } else {
            if body_id == BodyId::MidlanderFemale {
                return BodyId::MidlanderMale;
            }

            let search_id = if body_id == BodyId::LalafellFemale {
                BodyId::LalafellMale
            } else if body_id.is_child() {
                BodyId::ChildHyurMale
            } else if body_id == BodyId::HrothgarMale {
                BodyId::RoegadynMale
            } else if body_id.is_male() {
                BodyId::MidlanderMale
            } else {
                BodyId::MidlanderFemale
            };

            let eqdp = self.equipment_deformer_parameters.get(&search_id).unwrap();
            if eqdp.has_model(model_id, model_part as u8) {
                return search_id;
            }
            BodyId::MidlanderMale
        }
    }

    async fn create_empty_texture(renderer: &Renderer) -> Arc<Texture> {
        Arc::new(Texture::with_texels(renderer, 1, 1, &[0, 0, 0, 0], TextureFormat::Rgba8Unorm).await)
    }

    async fn create_equipment_deformer_parameters(package: &dyn Package) -> Result<HashMap<BodyId, Eqdp>> {
        BodyId::into_enum_iter()
            .map(|body_id| {
                Eqdp::new(
                    package,
                    format!("chara/xls/charadb/equipmentdeformerparameter/c{:04}.eqdp", body_id as u16),
                )
                .map(move |eqdp| Ok((body_id, eqdp?)))
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<HashMap<_, _>>()
            .await
    }
}