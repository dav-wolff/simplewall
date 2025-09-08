use std::path::Path;

use wayland_client::{
	Connection,
	QueueHandle,
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

mod wallpaper;
use wallpaper::Wallpaper;

fn main() {
	let wallpaper = Wallpaper::load(Path::new("wallpaper.jpg")).unwrap();
	
	let conn = Connection::connect_to_env().unwrap();
	
	let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
	let qh: QueueHandle<SimpleWall> = event_queue.handle();
	
	let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
	let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
	let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");
	
	let surface = compositor.create_surface(&qh);
	let layer = layer_shell.create_layer_surface(&qh, surface, Layer::Background, Some("wallpaper"), None);
	layer.set_anchor(Anchor::LEFT | Anchor::RIGHT | Anchor::TOP | Anchor::BOTTOM);
	layer.set_keyboard_interactivity(smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::None);
	layer.set_exclusive_zone(-1); // ignore other exclusive zones
	layer.commit();
	
	// TODO what does the len do?
	// can't know the resolution yet, but setting a smaller number seems to work just fine
	let pool = SlotPool::new(2560 * 1440 * 4, &shm).expect("failed to create pool");
	
	let mut simple_wall = SimpleWall {
		wallpaper,
		registry_state: RegistryState::new(&globals),
		output_state: OutputState::new(&globals, &qh),
		shm,
		pool,
		layer,
		closed: false,
		is_configured: false,
		width: 0,
		height: 0,
	};
	
	while !simple_wall.closed {
		event_queue.blocking_dispatch(&mut simple_wall).unwrap();
	}
}

struct SimpleWall {
	wallpaper: Wallpaper,
	registry_state: RegistryState,
	output_state: OutputState,
	shm: Shm,
	pool: SlotPool,
	layer: LayerSurface,
	closed: bool,
	is_configured: bool,
	width: u32,
	height: u32,
}

impl LayerShellHandler for SimpleWall {
	fn closed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_layer: &LayerSurface
	) {
		self.closed = true;
	}
	
	fn configure(
		&mut self,
		_conn: &Connection,
		qh: &QueueHandle<Self>,
		_layer: &LayerSurface,
		configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
		_serial: u32,
	) {
		(self.width, self.height) = configure.new_size;
		
		if !self.is_configured {
			self.is_configured = true;
			self.draw(qh);
		}
	}
}

impl SimpleWall {
	fn draw(&mut self, qh: &QueueHandle<Self>) {
		let stride = self.width as i32 * 4;
		
		let (buffer, canvas) = self.pool.create_buffer(self.width as i32, self.height as i32, stride, Format::Xrgb8888).unwrap();
		
		self.wallpaper.resize_into(self.width, self.height, canvas);
		
		self.layer.wl_surface().damage_buffer(0, 0, self.width as i32, self.height as i32);
		
		self.layer.wl_surface().frame(qh, self.layer.wl_surface().clone());
		
		buffer.attach_to(self.layer.wl_surface()).unwrap();
		self.layer.commit();
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
