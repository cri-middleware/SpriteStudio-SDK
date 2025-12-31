use crate::runtime;

fn get_part<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, index: u16, part: &ssab::ssab::PartAnimationData, frame_no: i32) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartState<'a>> {
    let mut p_args = runtime::framedata::ss::runtime::PartStateArgs::default();

    p_args.part_index = index;
    p_args.update_flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();
    part.cell_array().map(|cell_array| {
        // p_args.cell;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell;
    });
    part.pos_x_array().map(|pos_x_array| {
        // p_args.position_x;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX;
    });
    part.pos_y_array().map(|pos_y_array| {
        // p_args.position_y;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosY;
    });
    part.pos_z_array().map(|pos_z_array| {
        // p_args.position_z;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosZ;
    });
    part.rot_x_array().map(|rotation_x_array| {
        // p_args.rotation_x;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotX;
    });
    part.rot_y_array().map(|rotation_y_array| {
        // p_args.rotation_y;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotY;
    });
    part.rot_z_array().map(|rotation_z_array| {
        // p_args.rotation_z;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ;
    });
    part.scl_x_array().map(|scale_x_array| {
        // p_args.scale_x;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclX;
    });
    part.scl_y_array().map(|scale_y_array| {
        // p_args.scale_y;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclY;
    });
    part.loscl_x_array().map(|local_scale_x_array| {
        // p_args.local_scale_x;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclX;
    });
    part.loscl_y_array().map(|local_scale_y_array| {
        // p_args.local_scale_y;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclY;
    });
    part.alpha_array().map(|alpha_array| {
        // p_args.opacity;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAlpha;
    });
    part.lo_alpha_array().map(|local_alpha_array| {
        // p_args.localopacity;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLoAlpha;
    });
    part.prio_array().map(|priority_array| {
        // p_args.priority;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePrio;
    });
    part.flip_v_array().map(|flip_v_array| {
        // p_args.flip_v;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipV;
    });
    part.flip_h_array().map(|flip_h_array| {
        // p_args.flip_h;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipH;
    });
    part.hide_array().map(|hide_array| {
        // p_args.hide;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeHide;
    });
    part.parts_color_array().map(|parts_color_array| {
        // p_args.parts_color;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor;
    });
    part.shader_array().map(|shader_array| {
        // p_args.shader;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeShader;
    });
    part.vertex_array().map(|vertex_array| {
        // p_args.vertex;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex;
    });
    part.pivot_x_array().map(|pivot_x_array| {
        // p_args.pivot_x;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotX;
    });
    part.pivot_y_array().map(|pivot_y_array| {
        // p_args.pivot_y;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotY;
    });
    part.anchor_x_array().map(|anchor_x_array| {
        // p_args.anchor_x;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorX;
    });
    part.anchor_y_array().map(|anchor_y_array| {
        // p_args.anchor_y;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorY;
    });
    part.size_x_array().map(|size_x_array| {
        // p_args.size_x;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeX;
    });
    part.size_y_array().map(|size_y_array| {
        // p_args.size_y;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeY;
    });
    part.img_flip_h_array().map(|img_flip_h_array| {
        // p_args.img_flip_h;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipH;
    });
    part.img_flip_v_array().map(|img_flip_v_array| {
        // p_args.img_flip_v;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipV;
    });
    part.uvt_x_array().map(|uvt_x_array| {
        // p_args.uv_move_x;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtX;
    });
    part.uvt_y_array().map(|uvt_y_array| {
       // p_args.uv_move_y;
       // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtY;
    });
    part.uvr_z_array().map(|uvr_z_array| {
        // p_args.uv_move_z;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvrZ;
    });
    part.uvs_x_array().map(|uvs_x_array| {
        // p_args.uv_scale_x;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsX;
    });
    part.uvs_y_array().map(|uvs_y_array| {
        // p_args.uv_scale_y;
        // p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsY;
    });
    part.boundr_array().map(|boundr_array| {
        // p_args.bounding_radius;
    });
    part.mask_array().map(|mask_array| {
        // p_args.mask;
    });
    part.skew_array().map(|skew_array| {
        // p_args.skew_x;
        // p_args.skew_y;
    });
    part.deform_array().map(|deform_array| {
        // p_args.deform;
    });
    part.texture_array().map(|texture_array| {
        // p_args.texture;
    });

    runtime::framedata::ss::runtime::PartState::create(fb, &p_args)
}

fn get_parts<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, p: &flatbuffers::Vector<::flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationData>>, frame_no: i32) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartState<'a>>>> {
    let parts = p.iter().enumerate().map(|(index, part)| {
        get_part(fb, index as u16, &part, frame_no)
    }).collect::<Vec<_>>();
    fb.create_vector(&parts)
}

pub fn get_frame_data<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, animation: &ssab::ssab::AnimationData, frame_no: i32) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    let mut args = runtime::framedata::ss::runtime::FrameDataArgs::default();
    args.parts = animation.parts_animation_data().map( |parts_animation_data| get_parts(fb, &parts_animation_data, frame_no));

    runtime::framedata::ss::runtime::FrameData::create(fb, &args)
}
