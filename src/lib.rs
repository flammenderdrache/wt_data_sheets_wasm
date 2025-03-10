use std::f64;
use std::str::FromStr;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::prelude::*;
use web_sys::{console, Document, Window, window};
use wt_ballistics_calc_lib::launch_parameters::LaunchParameter;
use wt_ballistics_calc_lib::runner::{generate, LaunchResults, Splash};
use wt_missile_calc_lib::missiles::Missile;

use crate::table::make_table;

static STATIC_MISSILES: &str = include_str!("../../wt_missile_calc/index/all.json");

mod table;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
	// This provides better error messages in debug mode.
	// It's disabled in release mode so it doesn't bloat up the file size.
	#[cfg(debug_assertions)]
		console_error_panic_hook::set_once();

	let window: Window = web_sys::window().expect("no global `window` exists");
	let document: Document = window.document().expect("should have a document on window");
	let url: String = document.url().expect("should have a url");

	let url_local: &str = url.split("/").collect::<Vec<&str>>()[3];

	// Required as loading the functions via this module through the main is required (WASM doesnt support modules)
	match url_local {
		"" => { generate_main_tables(&document) }
		"live_calc.html" => { console_log("live") }
		_ => {}
	}


	#[wasm_bindgen]
	pub fn console_log(message: &str) {
		console::log_1(&JsValue::from_str(message));
	}

	#[wasm_bindgen]
	pub fn constant_calc(velocity: f64, alt: u32, missile_select: usize, do_splash: bool) {

		let mut attempted_distance = 10000.0;
		let mut parameters = LaunchParameter::new_from_parameters(false, (velocity / 3.6), attempted_distance, (velocity / 3.6), alt);

		let missiles: Vec<Missile> = serde_json::from_str(STATIC_MISSILES).unwrap();
		let mut results = generate(&missiles[missile_select], &parameters, 0.1, false);

		let window: Window = web_sys::window().expect("no global `window` exists");
		let document: Document = window.document().expect("should have a document on window");

		document.get_element_by_id("range").unwrap().set_inner_html(&results.distance_flown.round().to_string());

		attempted_distance = results.distance_flown.round();

		if do_splash {
			while !results.splash.splash {
				results = generate(&missiles[missile_select], &parameters, 0.1, false);
				parameters.distance_to_target -= 200.0;
			}
			document.get_element_by_id("splash_at").unwrap().set_inner_html(&results.splash.at.round().to_string());
		}else {
			document.get_element_by_id("splash_at").unwrap().set_inner_html("-");
		}
	}

	#[wasm_bindgen]
	pub fn make_option_inputs() {
		let window: Window = web_sys::window().expect("no global `window` exists");
		let document: Document = window.document().expect("should have a document on window");

		let missiles: Vec<Missile> = serde_json::from_str(STATIC_MISSILES).unwrap();

		let select = document.get_element_by_id("missile_select").unwrap();

		for (i, missile) in missiles.iter().enumerate() {
			let missile_element = document.create_element("option").unwrap();
			missile_element.set_attribute("value", &i.to_string());
			missile_element.set_text_content(Some(&missile.name));
			select.append_child(&missile_element);
		}
	}

	Ok(())
}

fn generate_main_tables(document: &web_sys::Document) {
	let mut parameters = LaunchParameter::new_from_parameters(false, 343.0, 0.0, 0.0, 0);

	let url: String = document.url().unwrap(); // gets url from entire page

	if url.contains("?") {
		let mut keys = "";

		console::log_1(&JsValue::from_str("Using custom values"));

		keys = url.split("?").collect::<Vec<&str>>()[1]; // separates url.com/?yes to ?yes

		let values = keys.split("+").collect::<Vec<&str>>(); // separates values like one=1+two=2

		for value in values {
			if value.contains("alt") {
				let parameer = &value.clone()[4..];
				parameters.altitude = u32::from_str(parameer).unwrap();
			}
			if value.contains("vel") {
				let parameer = &value.clone()[4..];
				parameters.start_velocity = f64::from_str(parameer).unwrap();
			}
		}
	}

	document.get_element_by_id("alt").unwrap().set_attribute("value", &parameters.altitude.to_string());
	document.get_element_by_id("vel").unwrap().set_attribute("value", &parameters.start_velocity.to_string());

	make_table(&parameters);
}