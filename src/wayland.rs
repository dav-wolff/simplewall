use std::collections::HashMap;

use wayland_client::{
	Connection,
	QueueHandle,
	Proxy,
	backend::ObjectId,
	globals::registry_queue_init,
	protocol::wl_shm::Format,
	protocol::wl_surface::WlSurface,
	protocol::wl_output::{Transform, WlOutput},
};
use smithay_client_toolkit::{
	delegate_compositor,
	delegate_registry,
	delegate_layer,
	delegate_shm,
	delegate_output,
	compositor::{CompositorHandler, CompositorState},
	registry::{ProvidesRegistryState, RegistryState},
	shell::WaylandSurface,
	shell::wlr_layer::{Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface},
	shm::{Shm, ShmHandler},
	shm::slot::SlotPool,
	output::{OutputHandler, OutputState},
};

use crate::{
	wallpaper::Wallpaper,
	WallpaperOptions
};

pub fn run(wallpapers: Vec<WallpaperOptions>) {
	let conn = Connection::connect_to_env().unwrap();
	
	let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
	let qh: QueueHandle<SimpleWall> = event_queue.handle();
	
	let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
	let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
	let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
	
	// TODO what does the len do?
	// can't know the resolution yet, but setting a smaller number seems to work just fine
	let pool = SlotPool::new(2560 * 1440 * 4 * wallpapers.len(), &shm).expect("failed to create pool");
	
	let mut wallpaper_surfaces = HashMap::new();
	
	for wallpaper_options in wallpapers {
		let surface = compositor.create_surface(&qh);
		let surface_id = surface.id();
		let layer_surface = layer_shell.create_layer_surface(&qh, surface, Layer::Background, Some(wallpaper_options.namespace), None);
		layer_surface.set_anchor(Anchor::LEFT | Anchor::RIGHT | Anchor::TOP | Anchor::BOTTOM);
		layer_surface.set_keyboard_interactivity(smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::None);
		layer_surface.set_exclusive_zone(-1); // ignore other exclusive zones
		layer_surface.commit();
		
		let wallpaper_surface = WallpaperSurface {
			wallpaper: wallpaper_options.wallpaper,
			surface: layer_surface,
		};
		
		wallpaper_surfaces.insert(surface_id, wallpaper_surface);
	}
	
	let mut simple_wall = SimpleWall {
		wallpapers: wallpaper_surfaces,
		registry_state: RegistryState::new(&globals),
		output_state: OutputState::new(&globals, &qh),
		shm,
		pool,
	};
	
	while !simple_wall.all_closed() {
		event_queue.blocking_dispatch(&mut simple_wall).unwrap();
	}
}

struct WallpaperSurface {
	wallpaper: Wallpaper,
	surface: LayerSurface,
}

struct SimpleWall {
	wallpapers: HashMap<ObjectId, WallpaperSurface>,
	registry_state: RegistryState,
	output_state: OutputState,
	shm: Shm,
	pool: SlotPool,
}

impl WallpaperSurface {
	fn draw(&mut self, (width, height): (u32, u32), pool: &mut SlotPool, qh: &QueueHandle<SimpleWall>) {
		let stride = width as i32 * 4;
		
		let (buffer, canvas) = pool.create_buffer(width as i32, height as i32, stride, Format::Xrgb8888).unwrap();
		
		self.wallpaper.resize_into(width, height, canvas);
		
		self.surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
		
		self.surface.wl_surface().frame(qh, self.surface.wl_surface().clone());
		
		buffer.attach_to(self.surface.wl_surface()).unwrap();
		self.surface.commit();
	}
}

impl SimpleWall {
	fn all_closed(&self) -> bool {
		self.wallpapers.is_empty()
	}
}

impl LayerShellHandler for SimpleWall {
	fn closed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		layer: &LayerSurface
	) {
		let id = layer.wl_surface().id();
		self.wallpapers.remove(&id);
	}
	
	fn configure(
		&mut self,
		_conn: &Connection,
		qh: &QueueHandle<Self>,
		layer: &LayerSurface,
		configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
		_serial: u32,
	) {
		let id = layer.wl_surface().id();
		let wallpaper = self.wallpapers.get_mut(&id).expect("Received configure event for surface with unknown id: {id}");
		wallpaper.draw(configure.new_size, &mut self.pool, qh);
		
		// TODO: why does the example do this? shouldn't the surface be redrawn every time it's resized? when does configure get called?
		// if !self.is_configured {
		// 	self.is_configured = true;
		// 	self.draw(qh);
		// }
	}
}

delegate_compositor!(SimpleWall);
delegate_registry!(SimpleWall);
delegate_layer!(SimpleWall);
delegate_output!(SimpleWall);
delegate_shm!(SimpleWall);

impl CompositorHandler for SimpleWall {
	fn scale_factor_changed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &WlSurface,
		_new_factor: i32,
	) { }
	
	fn transform_changed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &WlSurface,
		_new_transform: Transform,
	) { }
	
	fn frame(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &WlSurface,
		_time: u32,
	) { }
	
	fn surface_enter(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &WlSurface,
		_output: &WlOutput,
	) { }
	
	fn surface_leave(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &WlSurface,
		_output: &WlOutput,
	) { }
}

impl ProvidesRegistryState for SimpleWall {
	fn registry(&mut self) -> &mut RegistryState {
		&mut self.registry_state
	}
	
	fn runtime_add_global(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_name: u32,
		_interface: &str,
		_version: u32,
	) { }
	
	fn runtime_remove_global(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_name: u32,
		_interface: &str,
	) { }
}

impl OutputHandler for SimpleWall {
	fn output_state(&mut self) -> &mut OutputState {
		&mut self.output_state
	}
	
	fn new_output(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_output: WlOutput,
	) { } // TODO: is some extra work necessary to support multiple / hot swapping?
	
	fn update_output(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_output: WlOutput,
	) { }
	
	fn output_destroyed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_output: WlOutput,
	) { }
}

impl ShmHandler for SimpleWall {
	fn shm_state(&mut self) -> &mut Shm {
		&mut self.shm
	}
}
