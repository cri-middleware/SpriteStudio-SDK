use crate::runtime;

fn get_part<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, part: &ssab::ssab::PartAnimationData, frame_no: i32) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartState<'a>> {
    let mut p_args = runtime::framedata::ss::runtime::PartStateArgs::default();

    p_args.part_index;
    p_args.update_flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();
    p_args.cell;
    p_args.position_x;
    p_args.position_y;
    p_args.position_z;
    p_args.rotation_x;
    p_args.rotation_y;
    p_args.rotation_z;
    p_args.scale_x;
    p_args.scale_y;
    p_args.local_scale_x;
    p_args.local_scale_y;
    p_args.opacity;
    p_args.localopacity;
    p_args.priority;
    p_args.flip_v;
    p_args.flip_h;
    p_args.hide;
    p_args.parts_color;
    p_args.shader;
    p_args.vertex;
    p_args.pivot_x;
    p_args.pivot_y;
    p_args.anchor_x;
    p_args.anchor_y;
    p_args.size_x;
    p_args.size_y;
    p_args.img_flip_h;
    p_args.img_flip_v;
    p_args.uv_move_x;
    p_args.uv_move_y;
    p_args.uv_move_z;
    p_args.uv_scale_x;
    p_args.uv_scale_y;
    p_args.bounding_radius;
    p_args.mask;
    p_args.skew_x;
    p_args.skew_y;
    p_args.deform;
    p_args.texture;

    runtime::framedata::ss::runtime::PartState::create(fb, &p_args)
}

fn get_parts<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, p: &flatbuffers::Vector<::flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationData>>, frame_no: i32) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartState<'a>>>> {
    let parts = p.iter().map(|part| {
        get_part(fb, &part, frame_no)
    }).collect::<Vec<_>>();
    fb.create_vector(&parts)
}

pub fn get_frame_data<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, animation: &ssab::ssab::AnimationData, frame_no: i32) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    let mut args = runtime::framedata::ss::runtime::FrameDataArgs::default();
    args.parts = animation.parts_animation_data().map( |parts_animation_data| get_parts(fb, &parts_animation_data, frame_no));

    runtime::framedata::ss::runtime::FrameData::create(fb, &args)
}
