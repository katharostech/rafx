use crate::features::mesh::{MeshRenderFeature, ExtractedFrameNodeMeshData, PreparedSubmitNodeMeshData};
use renderer::nodes::{
    RenderFeatureIndex, RenderPhaseIndex, RenderFeature, SubmitNodeId, FeatureCommandWriter,
    RenderView,
};
use crate::render_contexts::RenderJobWriteContext;
use renderer::assets::resources::{DescriptorSetArc, ResourceArc, GraphicsPipelineResource};
use ash::vk;
use ash::version::DeviceV1_0;

pub struct MeshCommandWriter {
    pub(super) extracted_frame_node_mesh_data: Vec<Option<ExtractedFrameNodeMeshData>>,
    pub(super) prepared_submit_node_mesh_data: Vec<PreparedSubmitNodeMeshData>,
}

impl FeatureCommandWriter<RenderJobWriteContext> for MeshCommandWriter {
    fn apply_setup(
        &self,
        write_context: &mut RenderJobWriteContext,
        _view: &RenderView,
        _render_phase_index: RenderPhaseIndex,
    ) {
        // println!("render");
        // let logical_device = write_context.device_context.device();
        // let command_buffer = write_context.command_buffer;
        // unsafe {
        //     logical_device.cmd_bind_pipeline(
        //         command_buffer,
        //         vk::PipelineBindPoint::GRAPHICS,
        //         self.pipeline_info.get_raw().pipelines[0],
        //     );
        // }
    }

    fn render_element(
        &self,
        write_context: &mut RenderJobWriteContext,
        _view: &RenderView,
        _render_phase_index: RenderPhaseIndex,
        index: SubmitNodeId,
    ) {
        let logical_device = write_context.device_context.device();
        let command_buffer = write_context.command_buffer;

        let render_node_data = &self.prepared_submit_node_mesh_data[index as usize];
        let frame_node_data : &ExtractedFrameNodeMeshData = self.extracted_frame_node_mesh_data[render_node_data.frame_node_index as usize]
            .as_ref()
            .unwrap();

        unsafe {
            let mesh_part = &frame_node_data.mesh_asset.inner.mesh_parts[render_node_data.mesh_part_index];

            let pipeline = write_context
                .resource_context
                .graphics_pipeline_cache()
                .get_or_create_graphics_pipeline(
                    &mesh_part.material_passes[0].material_pass_resource,
                    &write_context.renderpass
                ).unwrap();

            logical_device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.get_raw().pipelines[write_context.subpass_index as usize],
            );

            // Bind per-pass data (UBO with view/proj matrix, sampler)
            logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline
                    .get_raw()
                    .pipeline_layout
                    .get_raw()
                    .pipeline_layout,
                0,
                &[render_node_data.per_view_descriptor_set.get()],
                &[],
            );

            // Bind per-draw-call data (i.e. texture)
            logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline
                    .get_raw()
                    .pipeline_layout
                    .get_raw()
                    .pipeline_layout,
                1,
                &[mesh_part.material_instance_descriptor_sets[0 /* pass index */][1 /* descriptor set index */].get()],
                &[],
            );

            logical_device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline
                    .get_raw()
                    .pipeline_layout
                    .get_raw()
                    .pipeline_layout,
                2,
                &[render_node_data.per_instance_descriptor_set.get()],
                &[],
            );

            logical_device.cmd_bind_vertex_buffers(
                command_buffer,
                0, // first binding
                &[frame_node_data.mesh_asset.inner.vertex_buffer.get_raw().buffer.buffer],
                &[mesh_part.vertex_buffer_offset_in_bytes as u64], // offsets
            );

            logical_device.cmd_bind_index_buffer(
                command_buffer,
                frame_node_data.mesh_asset.inner.index_buffer.get_raw().buffer.buffer,
                mesh_part.index_buffer_offset_in_bytes as u64, // offset
                vk::IndexType::UINT16,
            );

            logical_device.cmd_draw_indexed(
                command_buffer,
                mesh_part.index_buffer_size_in_bytes / 2, //sizeof(u16)
                1,
                0,
                0,
                0,
            );
        }
    }

    fn revert_setup(
        &self,
        _write_context: &mut RenderJobWriteContext,
        _view: &RenderView,
        _render_phase_index: RenderPhaseIndex,
    ) {
    }

    fn feature_debug_name(&self) -> &'static str {
        MeshRenderFeature::feature_debug_name()
    }

    fn feature_index(&self) -> RenderFeatureIndex {
        MeshRenderFeature::feature_index()
    }
}
