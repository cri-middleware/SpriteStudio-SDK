use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2Key {
    pub v1: ordered_float::OrderedFloat<f32>,
    pub v2: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::Vec2T> for Vec2Key {
    fn from(v: &ssab::ssab::Vec2T) -> Self {
        Self {
            v1: ordered_float::OrderedFloat(v.v1),
            v2: ordered_float::OrderedFloat(v.v2),
        }
    }
}
impl Vec2Key {
    fn new(x: f32, y: f32) -> Self {
        Self {
            v1: ordered_float::OrderedFloat(x),
            v2: ordered_float::OrderedFloat(y),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RectKey {
    pub x1: ordered_float::OrderedFloat<f32>,
    pub y1: ordered_float::OrderedFloat<f32>,
    pub x2: ordered_float::OrderedFloat<f32>,
    pub y2: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::RectT> for RectKey {
    fn from(value: &ssab::ssab::RectT) -> Self {
        Self {
            x1: ordered_float::OrderedFloat(value.x1),
            y1: ordered_float::OrderedFloat(value.y1),
            x2: ordered_float::OrderedFloat(value.x2),
            y2: ordered_float::OrderedFloat(value.y2),
        }
    }
}
impl RectKey {
    fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            x1: ordered_float::OrderedFloat(x1),
            y1: ordered_float::OrderedFloat(y1),
            x2: ordered_float::OrderedFloat(x2),
            y2: ordered_float::OrderedFloat(y2),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct IntValueEntryKey {
    pub value: i32,
}
impl From<&ssab::ssab::IntValueEntryT> for IntValueEntryKey {
    fn from(int_value_table_t: &ssab::ssab::IntValueEntryT) -> Self {
        Self {
            value: int_value_table_t.value,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct FloatValueEntryKey {
    pub value: ordered_float::OrderedFloat<f32>,
}
impl From<&ssab::ssab::FloatValueEntryT> for FloatValueEntryKey {
    fn from(float_value_table_t: &ssab::ssab::FloatValueEntryT) -> Self {
        Self {
            value: ordered_float::OrderedFloat(float_value_table_t.value),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct Vec2ValueEntryKey {
    pub value: Vec2Key,
}
impl From<&ssab::ssab::Vec2ValueEntryT> for Vec2ValueEntryKey {
    fn from(vec2_value_table_t: &ssab::ssab::Vec2ValueEntryT) -> Self {
        let value = vec2_value_table_t.value.clone().unwrap();
        let v1 = value.v1;
        let v2 = value.v2;
        Self {
            value: Vec2Key::new(v1, v2),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RectValueEntryKey {
    pub value: RectKey,
}
impl From<&ssab::ssab::RectValueEntryT> for RectValueEntryKey {
    fn from(rect_value_table_t: &ssab::ssab::RectValueEntryT) -> Self {
        let value = rect_value_table_t.value.clone().unwrap();
        let x1 = value.x1;
        let y1 = value.y1;
        let x2 = value.x2;
        let y2 = value.y2;
        Self {
            value: RectKey::new(x1, y1, x2, y2),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CellKey {
    pub name: String,
    pub rectangle: Option<RectKey>,
    pub pivot: Option<Vec2Key>,
    pub rotated: bool,
    pub table_coordinate: Option<Vec<Vec2Key>>,
    pub table_index_vertex: Option<Vec<u16>>,
}
impl From<&ssab::ssab::CellT> for CellKey {
    fn from(value: &ssab::ssab::CellT) -> Self {
        Self {
            name: value.name.clone(),
            rectangle: value.rectangle.as_ref().map(|r| RectKey::from(r.as_ref())),
            pivot: value.pivot.as_ref().map(|p| Vec2Key::from(p.as_ref())),
            rotated: value.rotated,
            table_coordinate: value.table_coordinate.as_ref().map(|t| t.iter().map(|v| Vec2Key::from(v)).collect::<Vec<Vec2Key>>()),
            table_index_vertex: value.table_index_vertex.as_ref().map(|t| t.iter().map(|v| *v).collect::<Vec<u16>>()),
        }
    }
}

pub struct SsabCache<'a> {
    pub vec2_cache: HashMap<Vec2Key, flatbuffers::WIPOffset<ssab::ssab::Vec2<'a>>>,
    pub rect_cache: HashMap<RectKey, flatbuffers::WIPOffset<ssab::ssab::Rect<'a>>>,
    pub int_value_table_cache: HashMap<IntValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::IntValueEntry<'a>>>,
    pub float_value_table_cache: HashMap<FloatValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::FloatValueEntry<'a>>>,
    pub vec2_value_table_cache: HashMap<Vec2ValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::Vec2ValueEntry<'a>>>,
    pub rect_value_table_cache: HashMap<RectValueEntryKey, flatbuffers::WIPOffset<ssab::ssab::RectValueEntry<'a>>>,

    pub cell_cache: HashMap<CellKey, flatbuffers::WIPOffset<ssab::ssab::Cell<'a>>>,

    pub part_type_none_cache: HashMap<String, flatbuffers::WIPOffset<ssab::ssab::PartTypeNone<'a>>>,
}
impl<'a> SsabCache<'a> {
    pub fn new() -> Self {
        Self {
            vec2_cache: HashMap::new(),
            rect_cache: HashMap::new(),
            int_value_table_cache: HashMap::new(),
            float_value_table_cache: HashMap::new(),
            vec2_value_table_cache: HashMap::new(),
            rect_value_table_cache: HashMap::new(),

            cell_cache: HashMap::new(),

            part_type_none_cache: HashMap::new(),
        }
    }
    fn create_shared_vec2(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, vec2t: &ssab::ssab::Vec2T) -> flatbuffers::WIPOffset<ssab::ssab::Vec2<'a>> {
        let key = Vec2Key::from(vec2t);
        if let Some(&offset) = self.vec2_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::Vec2Args::default();
        args.v1 = vec2t.v1;
        args.v2 = vec2t.v2;
        let offset = ssab::ssab::Vec2::create(fb, &args);
        self.vec2_cache.insert(key, offset);
        offset
    }

    fn create_shared_rect(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, rectt: &ssab::ssab::RectT) -> flatbuffers::WIPOffset<ssab::ssab::Rect<'a>> {
        let key = RectKey::from(rectt);
        if let Some(&offset) = self.rect_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::RectArgs::default();
        args.x1 = rectt.x1;
        args.y1 = rectt.y1;
        args.x2 = rectt.x2;
        args.y2 = rectt.y2;
        let offset = ssab::ssab::Rect::create(fb, &args);
        self.rect_cache.insert(key, offset);
        offset
    }

    #[allow(dead_code)]
    fn create_shared_int_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, int_value_table_t: &ssab::ssab::IntValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::IntValueEntry<'a>> {
        let key = IntValueEntryKey::from(int_value_table_t);
        if let Some(&offset) = self.int_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::IntValueEntryArgs::default();
        args.value = int_value_table_t.value;
        let offset = ssab::ssab::IntValueEntry::create(fb, &args);
        offset
    }

    #[allow(dead_code)]
    fn create_shared_float_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, float_value_table_t: &ssab::ssab::FloatValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::FloatValueEntry<'a>> {
        let key = FloatValueEntryKey::from(float_value_table_t);
        if let Some(&offset) = self.float_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::FloatValueEntryArgs::default();
        args.value = float_value_table_t.value;
        let offset = ssab::ssab::FloatValueEntry::create(fb, &args);
        offset
    }

    #[allow(dead_code)]
    fn create_shared_vec2_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, vec2_value_table_t: &ssab::ssab::Vec2ValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::Vec2ValueEntry<'a>> {
        let key = Vec2ValueEntryKey::from(vec2_value_table_t);
        if let Some(&offset) = self.vec2_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::Vec2ValueEntryArgs::default();
        let v = vec2_value_table_t.value.as_ref().unwrap();
        args.value = Some(self.create_shared_vec2(fb, &v));
        let offset = ssab::ssab::Vec2ValueEntry::create(fb, &args);
        offset
    }

    #[allow(dead_code)]
    fn create_shared_rect_value_table(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, rect_value_table_t: &ssab::ssab::RectValueEntryT) -> flatbuffers::WIPOffset<ssab::ssab::RectValueEntry<'a>> {
        let key = RectValueEntryKey::from(rect_value_table_t);
        if let Some(&offset) = self.rect_value_table_cache.get(&key) {
            return offset;
        }
        let mut args = ssab::ssab::RectValueEntryArgs::default();
        let v = rect_value_table_t.value.as_ref().unwrap();
        args.value = Some(self.create_shared_rect(fb, &v));
        let offset = ssab::ssab::RectValueEntry::create(fb, &args);
        offset
    }
}

impl<'a> SsabCache<'a> {
    fn create_shared_cell(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, cell: &ssab::ssab::CellT) -> flatbuffers::WIPOffset<ssab::ssab::Cell<'a>> {
        let key = CellKey::from(cell);
        if let Some(&offset) = self.cell_cache.get(&key) {
            return offset;
        }

        let mut args = ssab::ssab::CellArgs::default();
        args.name = Some(fb.create_shared_string(&cell.name));
        args.rectangle = cell.rectangle.as_ref().map(|r| self.create_shared_rect(fb, r));
        args.pivot = cell.pivot.as_ref().map(|p| self.create_shared_vec2(fb, p));
        args.rotated = cell.rotated;
        args.table_coordinate = cell.table_coordinate.as_ref().map(|p| {
            let vec = p.iter().map(|v| self.create_shared_vec2(fb, v)).collect::<Vec<_>>();
            fb.create_vector(&vec)
        });
        args.table_index_vertex = cell.table_index_vertex.as_ref().map(|p| fb.create_vector(&p));

        let offset = ssab::ssab::Cell::create(fb, &args);
        self.cell_cache.insert(key, offset);
        offset
    }

    fn create_shared_part_type(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, part_type: &ssab::ssab::PartTypeT) -> flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset> {
        match part_type {
            ssab::ssab::PartTypeT::PartTypeShape(v) => {
                /*
                let mut p_args = ssab::ssab::PartTypeShapeArgs::default();
                p_args.shape_type = v.shape_type;
                p_args.shape_mask = v.shape_mask;
                let offset = ssab::ssab::PartTypeShape::create(fb, &p_args);
                offset.as_union_value()
                 */
                v.pack(fb).as_union_value()
            },
            ssab::ssab::PartTypeT::PartTypeText(v) => {
                let mut p_args = ssab::ssab::PartTypeTextArgs::default();
                p_args.text = v.text.as_ref().map(|s| fb.create_shared_string(s));
                p_args.text_bitmap = v.text_bitmap;
                p_args.text_family = v.text_family.as_ref().map(|s| fb.create_shared_string(s));
                p_args.text_char_map = v.text_char_map.as_ref().map(|s| fb.create_shared_string(s));
                p_args.text_size = v.text_size;
                p_args.text_space = v.text_space;
                p_args.text_smooth = v.text_smooth;
                p_args.text_mask = v.text_mask;
                p_args.text_width = v.text_width;
                p_args.text_height = v.text_height;
                p_args.e_anchor = v.e_anchor;
                let offset = ssab::ssab::PartTypeText::create(fb, &p_args);
                offset.as_union_value()
            },
            ssab::ssab::PartTypeT::PartTypeNines(v) => {
                v.pack(fb).as_union_value()
            },
            ssab::ssab::PartTypeT::PartTypeInstance(v) => {
                let mut p_args = ssab::ssab::PartTypeInstanceArgs::default();
                p_args.ref_anime = v.ref_anime.as_ref().map(|s| fb.create_shared_string(s));
                let offset = ssab::ssab::PartTypeInstance::create(fb, &p_args);
                offset.as_union_value()
            },
            ssab::ssab::PartTypeT::PartTypeEffect(v) => {
                let mut p_args = ssab::ssab::PartTypeEffectArgs::default();
                p_args.ref_effect_name = v.ref_effect_name.as_ref().map(|s| fb.create_shared_string(s));
                let offset = ssab::ssab::PartTypeEffect::create(fb, &p_args);
                offset.as_union_value()
            },
            _ => {
                if let Some(&offset) = self.part_type_none_cache.get("dummy") {
                    return offset.as_union_value()
                };
                let p_args = ssab::ssab::PartTypeNoneArgs::default();
                let offset = ssab::ssab::PartTypeNone::create(fb, &p_args);
                self.part_type_none_cache.insert("dummy".to_string(), offset);
                offset.as_union_value()
            },
        }
    }

    fn create_shared_part_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, ps: &Vec<ssab::ssab::PartDataT>) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::PartData<'a>>>> {
        let parts = ps.iter().map(|part| {
            let mut args = ssab::ssab::PartDataArgs::default();
            args.name = part.name.as_ref().map(|s| fb.create_shared_string(s));
            args.parent_index = part.parent_index;
            args.bounds_type = part.bounds_type;
            args.blend_type = part.blend_type;
            args.part_type_type = part.part_type.part_type_type();
            args.part_type = Some(self.create_shared_part_type(fb, &part.part_type));

            ssab::ssab::PartData::create(fb, &args)
        }).collect::<Vec<_>>();
        fb.create_vector(&parts)
    }

    fn create_shared_events_per_frame(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, es: &[ssab::ssab::EventsPerFrameT]) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::EventsPerFrame<'a>>>> {
        let v = es.iter().map(|event| {
            let mut e_args = ssab::ssab::EventsPerFrameArgs::default();
            e_args.frame_index = event.frame_index;
            e_args.users = event.users.as_ref().map(|users| {
                let v = users.iter().map(|user| {
                    let mut u_args = ssab::ssab::EventUserDataArgs::default();
                    u_args.part_index = user.part_index;
                    u_args.value = user.value.as_ref().map(|value| {
                        let mut u = ssab::ssab::PartAttributeUserDataArgs::default();
                        u.integer = value.integer.as_ref().map(|integer| self.create_shared_int_value_table(fb, &integer));
                        u.rect = value.rect.as_ref().map(|rect| self.create_shared_rect(fb, &rect));
                        u.point = value.point.as_ref().map(|point| self.create_shared_vec2(fb, &point));
                        u.str = value.str.as_ref().map(|s| fb.create_shared_string(s));

                        ssab::ssab::PartAttributeUserData::create(fb, &u)
                    });
                    ssab::ssab::EventUserData::create(fb, &u_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });

            e_args.signals = event.signals.as_ref().map(|signals| {
               let v = signals.iter().map(|signal| {
                   let mut s_args = ssab::ssab::EventSignalArgs::default();
                   s_args.part_index = signal.part_index;
                   s_args.value = signal.value.as_ref().map(|value| {
                       let mut s = ssab::ssab::PartAttributeSignalArgs::default();
                       s.active = value.active;
                       s.command_id = value.command_id.as_ref().map(|s| fb.create_shared_string(s));
                       s.params = value.params.as_ref().map(|params| {
                           let v = params.iter().map(|param| {
                               let mut p_args = ssab::ssab::SignalCommandParamArgs::default();
                               p_args.param_id = param.param_id.as_ref().map(|param_id| fb.create_shared_string(param_id));
                               p_args.type_ = param.type_;
                               p_args.value = param.value;

                               ssab::ssab::SignalCommandParam::create(fb, &p_args)
                           }).collect::<Vec<_>>();
                           fb.create_vector(&v)
                       });

                       ssab::ssab::PartAttributeSignal::create(fb, &s)
                   });

                   ssab::ssab::EventSignal::create(fb, &s_args)
               }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            e_args.audios = event.audios.as_ref().map(|audios| {
                let v = audios.iter().map(|audio| {
                    let mut a_args = ssab::ssab::EventAudioArgs::default();
                    a_args.part_index = audio.part_index;
                    a_args.value = audio.value.as_ref().map(|value| {
                        let mut pa_args = ssab::ssab::PartAttributeAudioArgs::default();
                        pa_args.sound_list_id = value.sound_list_id;
                        pa_args.sound_name = value.sound_name.as_ref().map(|sound_name| fb.create_shared_string(sound_name));
                        pa_args.loop_num = value.loop_num;

                        ssab::ssab::PartAttributeAudio::create(fb, &pa_args)
                    });
                    ssab::ssab::EventAudio::create(fb, &a_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            e_args.instances = event.instances.as_ref().map(|instances| {
                let v = instances.iter().map(|instance| {
                    let mut i_args = ssab::ssab::EventInstanceArgs::default();
                    i_args.part_index = instance.part_index;
                    i_args.value = instance.value.as_ref().map(|value| {
                        let mut pi_args = ssab::ssab::PartAttributeInstanceArgs::default();
                        pi_args.pingpong = value.pingpong;
                        pi_args.independent = value.independent;
                        pi_args.loop_num = value.loop_num;
                        pi_args.start_label = value.start_label.as_ref().map(|start_label| fb.create_shared_string(start_label));
                        pi_args.start_offset = value.start_offset;
                        pi_args.end_label = value.end_label.as_ref().map(|end_label| fb.create_shared_string(end_label));
                        pi_args.end_offset = value.end_offset;
                        pi_args.speed = value.speed;
                        ssab::ssab::PartAttributeInstance::create(fb, &pi_args)
                    });

                    ssab::ssab::EventInstance::create(fb, &i_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            e_args.effects = event.effects.as_ref().map(|effects| {
                let v = effects.iter().map(|effect| {
                    let mut e_args = ssab::ssab::EventEffectArgs::default();
                    e_args.part_index = effect.part_index;
                    e_args.value = effect.value.as_ref().map(|value| {
                        let mut pe_args = ssab::ssab::PartAttributeEffectArgs::default();
                        pe_args.cur_keyframe = value.cur_keyframe;
                        pe_args.start_time = value.start_time;
                        pe_args.speed = value.speed;
                        pe_args.loopflag = value.loopflag;
                        ssab::ssab::PartAttributeEffect::create(fb, &pe_args)
                    });

                    ssab::ssab::EventEffect::create(fb, &e_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });

            ssab::ssab::EventsPerFrame::create(fb, &e_args)
        }).collect::<Vec<_>>();
        fb.create_vector(&v)
    }

    fn create_animation_data_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, animation_data_t_vec: &[ssab::ssab::AnimationDataT]) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::AnimationData<'a>>>> {
        let animation_data_vec = animation_data_t_vec.iter().map(|animation_data_t| {
            let mut args = ssab::ssab::AnimationDataArgs::default();
            args.name = Some(fb.create_shared_string(&animation_data_t.name));
            args.parts_initial_attributes = None; // TODO; impl
            args.parts_animation_data = None; // TODO; impl
            args.parts_mesh_data = None; // TODO: impl
            args.events = animation_data_t.events.as_ref().map(|events| { self.create_shared_events_per_frame(fb, events) });
            args.labels = animation_data_t.labels.as_ref().map(|labels| {
                let v = labels.iter().map(|label| {
                    let mut l_args = ssab::ssab::LabelArgs::default();
                    l_args.name = Some(fb.create_shared_string(&label.name));
                    l_args.time = label.time;

                    ssab::ssab::Label::create(fb, &l_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            args.start_frame = animation_data_t.start_frame;
            args.end_frame = animation_data_t.end_frame;
            args.total_frame = animation_data_t.total_frame;
            args.fps = animation_data_t.fps;
            args.canvas = animation_data_t.canvas.as_ref().map(|c| self.create_shared_rect(fb, c));

            ssab::ssab::AnimationData::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&animation_data_vec)
    }

    fn create_shared_cell_map(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, cell_map_t_vec: &[ssab::ssab::CellMapT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::CellMap<'a>>>> {
        let cell_map = cell_map_t_vec.iter().map(|cell_map_t| {
            let mut args = ssab::ssab::CellMapArgs::default();
            args.name = Some(fb.create_shared_string(&cell_map_t.name));
            args.image_path = cell_map_t.image_path.as_ref().map(|s| fb.create_shared_string(s));
            args.size_original = cell_map_t.size_original.as_ref().map(|s| self.create_shared_vec2(fb, s));
            args.wrap_mode = cell_map_t.wrap_mode;
            args.filter_mode = cell_map_t.filter_mode;
            args.cells = cell_map_t.cells.as_ref().map(|c| {
                let a = c.iter().map(|cell| self.create_shared_cell(fb, cell)).collect::<Vec<_>>();
                fb.create_vector(&a)
            });

            ssab::ssab::CellMap::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&cell_map)
    }

    fn create_shared_sound_files(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, table_data_t_vec: &[ssab::ssab::SoundFileT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::SoundFile<'a>>>> {
        let table_data = table_data_t_vec.iter().map(|table_data_t| {
            let mut sound_file_args = ssab::ssab::SoundFileArgs::default();
            sound_file_args.name = Some(fb.create_shared_string(&table_data_t.name));
            sound_file_args.time_total = table_data_t.time_total;

            ssab::ssab::SoundFile::create(fb, &sound_file_args)
        }).collect::<Vec<_>>();

        fb.create_vector(&table_data)
    }

    fn create_shared_effect_node_behavior(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, behavior: &ssab::ssab::EffectNodeBehaviorT) -> Option<flatbuffers::WIPOffset<::flatbuffers::UnionWIPOffset>> {
        match behavior {
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementBasic(v) => {
                Some(v.pack(fb).as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementRndSeedChange(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementDelay(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleInfiniteEmitEnabled(v) => {
                let a = IntValueEntryKey::from(v.as_ref());
                if let Some(offset) = self.int_value_table_cache.get(&a) {
                    return Some(offset.as_union_value());
                };
                let mut w_args = ssab::ssab::IntValueEntryArgs::default();
                w_args.value = v.value;
                let offset = ssab::ssab::IntValueEntry::create(fb, &w_args);
                self.int_value_table_cache.insert(a, offset);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementGravity(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTransSpeed(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTangentialAcceleration(v) |
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementAlphaFade(v) => {
                let a = Vec2ValueEntryKey::from(v.as_ref());
                if let Some(offset) = self.vec2_value_table_cache.get(&a) {
                    return Some(offset.as_union_value());
                };
                let mut w_args = ssab::ssab::Vec2ValueEntryArgs::default();
                w_args.value = v.value.as_ref().map(|value| self.create_shared_vec2(fb, value));
                let offset = ssab::ssab::Vec2ValueEntry::create(fb, &w_args);
                self.vec2_value_table_cache.insert(a, offset);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementPosition(v) => {
                let a = RectValueEntryKey::from(v.as_ref());
                if let Some(offset) = self.rect_value_table_cache.get(&a) {
                    return Some(offset.as_union_value());
                };
                let mut w_args = ssab::ssab::RectValueEntryArgs::default();
                w_args.value = v.value.as_ref().map(|value| self.create_shared_rect(fb, value));
                let offset = ssab::ssab::RectValueEntry::create(fb, &w_args);
                self.rect_value_table_cache.insert(a, offset);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementRotation(v) => {
                let mut w_args = ssab::ssab::EffectParticleElementRotationArgs::default();
                w_args.rotation_value = v.rotation_value.as_ref().map(|rotation_value| self.create_shared_vec2(fb, &rotation_value));
                w_args.rotation_add_value = v.rotation_add_value.as_ref().map(|rotation_add_value| self.create_shared_vec2(fb, &rotation_add_value));
                let offset = ssab::ssab::EffectParticleElementRotation::create(fb, &w_args);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementRotationTrans(v) => {
                Some(v.pack(fb).as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementInitColor(v) => {
                Some(v.pack(fb).as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTransColor(v) => {
                Some(v.pack(fb).as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementSize(v) => {
                let mut w_args = ssab::ssab::RectAndVec2ValueTableArgs::default();
                w_args.size = v.size.as_ref().map(|size| self.create_shared_rect(fb, size));
                w_args.scale_factor_value = v.scale_factor_value.as_ref().map(|scale_factor_value| self.create_shared_vec2(fb, &scale_factor_value));
                let offset = ssab::ssab::RectAndVec2ValueTable::create(fb, &w_args);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTransSize(v) => {
                let mut w_args = ssab::ssab::RectAndVec2ValueTableArgs::default();
                w_args.size = v.size.as_ref().map(|size| self.create_shared_rect(fb, size));
                w_args.scale_factor_value = v.scale_factor_value.as_ref().map(|scale_factor_value| self.create_shared_vec2(fb, &scale_factor_value));
                let offset = ssab::ssab::RectAndVec2ValueTable::create(fb, &w_args);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticlePointGravity(v) => {
                let mut w_args = ssab::ssab::EffectParticlePointGravityArgs::default();
                w_args.position = v.position.as_ref().map(|position| self.create_shared_vec2(fb, position));
                w_args.power = v.power;
                let offset = ssab::ssab::EffectParticlePointGravity::create(fb, &w_args);
                Some(offset.as_union_value())
            },
            ssab::ssab::EffectNodeBehaviorT::EffectParticleTurnToDirectionEnabled(v) => {
                let a = FloatValueEntryKey::from(v.as_ref());
                if let Some(offset) = self.float_value_table_cache.get(&a) {
                    return Some(offset.as_union_value());
                };
                let mut w_args = ssab::ssab::FloatValueEntryArgs::default();
                w_args.value = v.value;
                let offset = ssab::ssab::FloatValueEntry::create(fb, &w_args);
                self.float_value_table_cache.insert(a, offset);
                Some(offset.as_union_value())
            },
            _ => None,
        }
    }

    fn create_shared_effect_nodes(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, ns: &[ssab::ssab::EffectNodeT], ) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::EffectNode<'a>>>> {
        let nodes = ns.iter().map(|node| {
            let mut n_args = ssab::ssab::EffectNodeArgs::default();
            n_args.index = node.index;
            n_args.parent_index = node.parent_index;
            n_args.effect_node_type = node.effect_node_type;
            n_args.cell_index = node.cell_index;
            n_args.blend_type = node.blend_type;
            n_args.behavior = node.behavior.as_ref().map(|behavior| {
                let v = behavior.iter().map(|b| {
                    let mut w_args = ssab::ssab::EffectNodeBehaviorWrapperArgs::default();
                    w_args.value_type = b.value.effect_node_behavior_type();
                    w_args.value = self.create_shared_effect_node_behavior(fb, &b.value);
                    ssab::ssab::EffectNodeBehaviorWrapper::create(fb, &w_args)
                }).collect::<Vec<_>>();
                fb.create_vector(&v)
            });
            ssab::ssab::EffectNode::create(fb, &n_args)
        }).collect::<Vec<_>>();
        fb.create_vector(&nodes)
    }

    fn create_shared_effect_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, effet_list_t_vec: &[ssab::ssab::EffectT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::Effect<'a>>>> {
        let sound_list = effet_list_t_vec.iter().map(|effect_t| {
            let mut args = ssab::ssab::EffectArgs::default();
            args.name = Some(fb.create_shared_string(&effect_t.name));
            args.fps = effect_t.fps;
            args.is_lock_rand_seed = effect_t.is_lock_rand_seed;
            args.lock_rand_seed = effect_t.lock_rand_seed;
            args.layout_scale = effect_t.layout_scale.as_ref().map(|v| self.create_shared_vec2(fb, v));
            args.effect_node = effect_t.effect_node.as_ref().map(|nodes| self.create_shared_effect_nodes(fb, &nodes));

            ssab::ssab::Effect::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&sound_list)
    }

    fn create_shared_sound_list(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, sound_list_t_vec: &[ssab::ssab::SoundListT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::SoundList<'a>>>> {
        let sound_list = sound_list_t_vec.iter().map(|sound_list_t| {
            let mut args = ssab::ssab::SoundListArgs::default();
            args.name = Some(fb.create_shared_string(&sound_list_t.name));
            args.table_data = sound_list_t.table_data.as_ref().map(|table_data| self.create_shared_sound_files(fb, table_data));

            ssab::ssab::SoundList::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&sound_list)
    }

    fn create_shared_font_bitmap(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, font_bitmap_t_vec: &[ssab::ssab::FontBitmapT], ) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::FontBitmap<'a>>>> {
        let font_bitmap = font_bitmap_t_vec.iter().map(|font_bitmap_t| {
            let mut args = ssab::ssab::FontBitmapArgs::default();
            args.name = Some(fb.create_shared_string(&font_bitmap_t.name));
            args.name_face = font_bitmap_t.name_face.as_ref().map(|s| fb.create_shared_string(s));
            args.size = font_bitmap_t.size;
            args.padding_l = font_bitmap_t.padding_l;
            args.padding_r = font_bitmap_t.padding_r;
            args.padding_u = font_bitmap_t.padding_u;
            args.padding_d = font_bitmap_t.padding_d;
            args.base = font_bitmap_t.base;
            args.height_line = font_bitmap_t.height_line;
            args.scale = font_bitmap_t.scale.as_ref().map(|s| self.create_shared_vec2(fb, s));
            args.spacing = font_bitmap_t.spacing.as_ref().map(|s| self.create_shared_vec2(fb, s));
            args.size_texture_original = font_bitmap_t.size_texture_original.as_ref().map(|s| self.create_shared_vec2(fb, s));
            args.name_texture = font_bitmap_t.name_texture.as_ref().map(|s| {
                let a = s.iter().map(|s| fb.create_shared_string(s)).collect::<Vec<_>>();
                fb.create_vector(&a)
            });
            args.table_data = None; // TODO: implement

            ssab::ssab::FontBitmap::create(fb, &args)
        }).collect::<Vec<_>>();

        fb.create_vector(&font_bitmap)
    }

    pub fn create_shared_anime_binary(&mut self, fb: &mut flatbuffers::FlatBufferBuilder<'a>, ss_anime_binary_t: &ssab::ssab::SsAnimeBinaryT, ) -> flatbuffers::WIPOffset<ssab::ssab::SsAnimeBinary<'a>> {
        let mut args = ssab::ssab::SsAnimeBinaryArgs::default();

        args.name = ss_anime_binary_t.name.as_ref().map(|s| fb.create_shared_string(s));
        args.parts = ss_anime_binary_t.parts.as_ref().map(|parts| self.create_shared_part_list(fb, parts));
        args.animations = ss_anime_binary_t.animations.as_ref().map(|animations| self.create_animation_data_list(fb, animations));
        args.cellmaps = ss_anime_binary_t.cellmaps.as_ref().map(|cell_map| self.create_shared_cell_map(fb, cell_map));
        args.effects = ss_anime_binary_t.effects.as_ref().map(|effect| self.create_shared_effect_list(fb, effect));
        args.sound_lists = ss_anime_binary_t.sound_lists.as_ref().map(|sound_list| self.create_shared_sound_list(fb, sound_list));
        args.font_bitmaps = ss_anime_binary_t.font_bitmaps.as_ref().map(|font_bitmap| self.create_shared_font_bitmap(fb, font_bitmap));
        args.external_instances = ss_anime_binary_t.external_instances.as_ref().map(|external_instances| {
            let vec = external_instances.iter().map(|s| fb.create_shared_string(s)).collect::<Vec<_>>();
            fb.create_vector(&vec)
        });

        ssab::ssab::SsAnimeBinary::create(fb, &args)
    }
}
