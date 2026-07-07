fn init(
    base_surface: &wl_surface::WlSurface,
    compositor: &Attached<wl_compositor::WlCompositor>,
    subcompositor: &Attached<wl_subcompositor::WlSubcompositor>,
    shm: &Attached<wl_shm::WlShm>,
    theme_manager: Option<ThemeManager>,
    implementation: Box<dyn FnMut(FrameRequest, u32, DispatchData)>,
) -> Result<ConceptFrame, ::std::io::Error> {
    let (themer, theme_over_surface) = if let Some(theme_manager) = theme_manager {
        (theme_manager, false)
    } else {
        (make_theme_manager(compositor.clone(), shm.clone()), true)
    };

    let inner = Rc::new(RefCell::new(Inner {
        parts: vec![],
        size: (1, 1),
        resizable: true,
        implem: implementation,
        theme_over_surface,
        maximized: false,
        fullscreened: false,
    }));

    let my_inner = inner.clone();
    // Send a Refresh request on callback from DoubleMemPool as it will be fired when
    // None was previously returned from `pool()` and the draw was postponed
    let pools = DoubleMemPool::new(shm.clone(), move |ddata| {
        (&mut my_inner.borrow_mut().implem)(FrameRequest::Refresh, 0, ddata);
    })?;

    Ok(ConceptFrame {
        base_surface: base_surface.clone(),
        compositor: compositor.clone(),
        subcompositor: subcompositor.clone(),
        inner,
        pools,
        active: WindowState::Inactive,
        hidden: true,
        pointers: Vec::new(),
        themer,
        surface_version: compositor.as_ref().version(),
        config: ConceptConfig::default(),
        title: None,
        shaped_title: None,
    })
}

fn new_seat(&mut self, seat: &Attached<wl_seat::WlSeat>) {
    use self::wl_pointer::Event;
    let inner = self.inner.clone();
    let pointer = self.themer.theme_pointer_with_impl(
        seat,
        move |event, pointer: ThemedPointer, ddata: DispatchData| {
            let data: &RefCell<PointerUserData> = pointer.as_ref().user_data().get().unwrap();
            let mut data = data.borrow_mut();
            let mut inner = inner.borrow_mut();
            match event {
                Event::Enter {
                    serial,
                    surface,
                    surface_x,
                    surface_y,
                } => {
                    data.location = precise_location(
                        inner.find_surface(&surface),
                        inner.size.0,
                        surface_x,
                        surface_y,
                    );
                    data.position = (surface_x, surface_y);
                    change_pointer(&pointer, &inner, data.location, Some(serial))
                }
                Event::Leave { serial, .. } => {
                    data.location = Location::None;
                    change_pointer(&pointer, &inner, data.location, Some(serial));
                    (&mut inner.implem)(FrameRequest::Refresh, 0, ddata);
                }
                Event::Motion {
                    surface_x,
                    surface_y,
                    ..
                } => {
                    data.position = (surface_x, surface_y);
                    let newpos =
                        precise_location(data.location, inner.size.0, surface_x, surface_y);
                    if newpos != data.location {
                        match (newpos, data.location) {
                            (Location::Button(_), _) | (_, Location::Button(_)) => {
                                // pointer movement involves a button, request refresh
                                (&mut inner.implem)(FrameRequest::Refresh, 0, ddata);
                            }
                            _ => (),
                        }
                        // we changed of part of the decoration, pointer image
                        // may need to be changed
                        data.location = newpos;
                        change_pointer(&pointer, &inner, data.location, None)
                    }
                }
                Event::Button {
                    serial,
                    button,
                    state,
                    ..
                } => {
                    if state == wl_pointer::ButtonState::Pressed {
                        let request = match button {
                            // Left mouse button.
                            0x110 => {
                                request_for_location_on_lmb(&data, inner.maximized, inner.resizable)
                            }
                            // Right mouse button.
                            0x111 => request_for_location_on_rmb(&data),
                            _ => None,
                        };

                        if let Some(request) = request {
                            (&mut inner.implem)(request, serial, ddata);
                        }
                    }
                }
                _ => {}
            }
        },
    );
    pointer.as_ref().user_data().set(|| {
        RefCell::new(PointerUserData {
            location: Location::None,
            position: (0.0, 0.0),
            seat: seat.detach(),
        })
    });
    self.pointers.push(pointer);
}

fn remove_seat(&mut self, seat: &wl_seat::WlSeat) {
    self.pointers.retain(|pointer| {
        let user_data = pointer
            .as_ref()
            .user_data()
            .get::<RefCell<PointerUserData>>()
            .unwrap();
        let guard = user_data.borrow_mut();
        if &guard.seat == seat {
            pointer.release();
            false
        } else {
            true
        }
    });
}

fn set_states(&mut self, states: &[State]) -> bool {
    let mut inner = self.inner.borrow_mut();
    let mut need_redraw = false;

    // Process active.
    let new_active = if states.contains(&State::Activated) {
        WindowState::Active
    } else {
        WindowState::Inactive
    };
    need_redraw |= new_active != self.active;
    self.active = new_active;

    // Process maximized.
    let new_maximized = states.contains(&State::Maximized);
    need_redraw |= new_maximized != inner.maximized;
    inner.maximized = new_maximized;

    // Process fullscreened.
    let new_fullscreened = states.contains(&State::Fullscreen);
    need_redraw |= new_fullscreened != inner.fullscreened;
    inner.fullscreened = new_fullscreened;

    need_redraw
}

fn set_hidden(&mut self, hidden: bool) {
    self.hidden = hidden;
    let mut inner = self.inner.borrow_mut();
    if !self.hidden {
        if inner.parts.is_empty() {
            inner.parts = vec![
                Part::new(
                    &self.base_surface,
                    &self.compositor,
                    &self.subcompositor,
                    Some(Rc::clone(&self.inner)),
                ),
                Part::new(
                    &self.base_surface,
                    &self.compositor,
                    &self.subcompositor,
                    None,
                ),
                Part::new(
                    &self.base_surface,
                    &self.compositor,
                    &self.subcompositor,
                    None,
                ),
                Part::new(
                    &self.base_surface,
                    &self.compositor,
                    &self.subcompositor,
                    None,
                ),
                Part::new(
                    &self.base_surface,
                    &self.compositor,
                    &self.subcompositor,
                    None,
                ),
            ];
        }
    } else {
        inner.parts.clear();
    }
}

fn set_resizable(&mut self, resizable: bool) {
    self.inner.borrow_mut().resizable = resizable;
}

fn resize(&mut self, newsize: (u32, u32)) {
    self.inner.borrow_mut().size = newsize;
}
