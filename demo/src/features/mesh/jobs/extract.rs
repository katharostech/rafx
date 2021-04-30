use rafx::render_feature_extract_job_predule::*;

use super::{
    ExtractedDirectionalLight, ExtractedFrameNodeMeshData, ExtractedPointLight, ExtractedSpotLight,
    MeshPrepareJob, MeshRenderNode, MeshRenderNodeSet, MeshStaticResources,
};
use crate::components::{
    DirectionalLightComponent, PointLightComponent, SpotLightComponent, TransformComponent,
};
use legion::*;
use rafx::assets::AssetManagerRenderResource;
use rafx::base::slab::RawSlabKey;

pub struct MeshExtractJob {}

impl MeshExtractJob {
    pub fn new() -> Self {
        Self {}
    }
}

impl ExtractJob for MeshExtractJob {
    fn extract(
        self: Box<Self>,
        extract_context: &RenderJobExtractContext,
        frame_packet: &FramePacket,
        _views: &[RenderView],
    ) -> Box<dyn PrepareJob> {
        profiling::scope!(super::EXTRACT_SCOPE_NAME);

        let legion_world = extract_context.extract_resources.fetch::<World>();
        let world = &*legion_world;

        let asset_manager = extract_context
            .render_resources
            .fetch::<AssetManagerRenderResource>();

        //
        // Update the mesh render nodes. This could be done earlier as part of a system
        //
        let mut mesh_render_nodes = extract_context
            .extract_resources
            .fetch_mut::<MeshRenderNodeSet>();
        mesh_render_nodes.update();

        //
        // Get the position/mesh asset pairs we will draw
        //
        let mut extracted_frame_node_mesh_data =
            Vec::<Option<ExtractedFrameNodeMeshData>>::with_capacity(
                frame_packet.frame_node_count(self.feature_index()) as usize,
            );

        for frame_node in frame_packet.frame_nodes(self.feature_index()).iter() {
            let entity_id = frame_node.entity_id();
            let entry = world.entry_ref(entity_id.into()).unwrap();
            let transform_component = entry.get_component::<TransformComponent>().unwrap();

            let render_node_index = frame_node.render_node_index();
            let render_node_handle = RawSlabKey::<MeshRenderNode>::new(render_node_index);

            let mesh_render_node = mesh_render_nodes
                .meshes
                .get_raw(render_node_handle)
                .unwrap();

            let mesh_asset = asset_manager.committed_asset(&mesh_render_node.mesh);

            let extracted_frame_node = mesh_asset.and_then(|mesh_asset| {
                Some(ExtractedFrameNodeMeshData {
                    mesh_asset: mesh_asset.clone(),
                    world_transform: glam::Mat4::from_scale_rotation_translation(
                        transform_component.scale,
                        transform_component.rotation,
                        transform_component.translation,
                    ),
                })
            });

            extracted_frame_node_mesh_data.push(extracted_frame_node);
        }

        //
        // Get the lights
        //
        let mut query = <(Entity, Read<DirectionalLightComponent>)>::query();
        let directional_lights = query
            .iter(world)
            .map(|(e, l)| ExtractedDirectionalLight {
                entity: *e,
                light: l.clone(),
            })
            .collect();

        let mut query = <(Entity, Read<TransformComponent>, Read<PointLightComponent>)>::query();
        let point_lights = query
            .iter(world)
            .map(|(e, p, l)| ExtractedPointLight {
                entity: *e,
                light: l.clone(),
                position: p.clone(),
            })
            .collect();

        let mut query = <(Entity, Read<TransformComponent>, Read<SpotLightComponent>)>::query();
        let spot_lights = query
            .iter(world)
            .map(|(e, p, l)| ExtractedSpotLight {
                entity: *e,
                light: l.clone(),
                position: p.clone(),
            })
            .collect();

        let static_resources = extract_context
            .render_resources
            .fetch::<MeshStaticResources>();

        let depth_material = asset_manager
            .committed_asset(&static_resources.depth_material)
            .unwrap()
            .get_single_material_pass()
            .unwrap();

        Box::new(MeshPrepareJob::new(
            depth_material,
            extracted_frame_node_mesh_data,
            directional_lights,
            point_lights,
            spot_lights,
        ))
    }

    fn feature_debug_name(&self) -> &'static str {
        super::render_feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        super::render_feature_index()
    }
}