//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::{entry, hal::gpio::FunctionSpi};
use defmt::*;
use defmt_rtt as _;
use embedded_graphics::{
    draw_target::DrawTarget,
    mono_font::MonoTextStyleBuilder,
    prelude::*,
    primitives::PrimitiveStyleBuilder,
    text::{Baseline, Text, TextStyleBuilder},
    Drawable,
};
use embedded_hal::digital::v2::OutputPin;
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use bsp::hal;
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use cortex_m::prelude::*;
use fugit::RateExtU32;

use epd_waveshare::{epd1in54_v2::Display1in54, epd1in54_v2::Epd1in54, prelude::*};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac: pac::Peripherals = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let _ = pins.gpio2.into_mode::<FunctionSpi>();
    let _ = pins.gpio3.into_mode::<FunctionSpi>();
    let _ = pins.gpio4.into_mode::<FunctionSpi>();

    let spi = hal::spi::Spi::<_, _, 8>::new(pac.SPI0);

    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        //4.MHz(),
        //4_000_000u32.Hz(),
        115_200u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );
    info!("Got SPI");

    let mut cs_pin = pins.gpio11.into_push_pull_output();
    cs_pin.set_high().unwrap();

    let busy_in = pins.gpio9.into_pull_up_input();
    let dc = pins.gpio8.into_push_pull_output();
    let mut rst = pins.gpio6.into_push_pull_output();
    info!("Got pins");

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    info!("Got delay");

    rst.set_high().unwrap();
    // Setup EPD
    let mut epd = Epd1in54::new(&mut spi, cs_pin, busy_in, dc, rst, &mut delay, None).unwrap();
    info!("made epd");
    epd.set_lut(&mut spi, &mut delay, Some(RefreshLut::Quick))
        .unwrap();
    // Use display graphics from embedded-graphics
    let _display = Display1in54::default();
    info!("made display");

    info!("Constructed display");

    // Use embedded graphics for drawing a line
    epd.wake_up(&mut spi, &mut delay).unwrap();
    info!("woke display");

    // Clear the full screen
    epd.clear_frame(&mut spi, &mut delay).unwrap();
    epd.display_frame(&mut spi, &mut delay).unwrap();
    info!("cleared an initial display");

    // Speeddemo
    epd.set_lut(&mut spi, &mut delay, Some(RefreshLut::Quick))
        .unwrap();
    let small_buffer = [Color::Black.get_byte_value(); 32]; //16x16
    let number_of_runs = 1;
    for i in 0..number_of_runs {
        let offset = i * 8 % 150;
        epd.update_partial_frame(
            &mut spi,
            &mut delay,
            &small_buffer,
            25 + offset,
            25 + offset,
            16,
            16,
        )
        .unwrap();
        epd.display_frame(&mut spi, &mut delay).unwrap();
    }

    // Clear the full screen
    epd.clear_frame(&mut spi, &mut delay).unwrap();
    epd.display_frame(&mut spi, &mut delay).unwrap();

    // Draw some squares

    let small_buffer = [Color::Black.get_byte_value(); 3200]; //160x160
    epd.update_partial_frame(&mut spi, &mut delay, &small_buffer, 20, 20, 160, 160)
        .unwrap();

    let small_buffer = [Color::White.get_byte_value(); 800]; //80x80
    epd.update_partial_frame(&mut spi, &mut delay, &small_buffer, 60, 60, 80, 80)
        .unwrap();

    let small_buffer = [Color::Black.get_byte_value(); 8]; //8x8
    epd.update_partial_frame(&mut spi, &mut delay, &small_buffer, 96, 96, 8, 8)
        .unwrap();

    // Display updated frame
    epd.display_frame(&mut spi, &mut delay).unwrap();
    delay.delay_ms(5000u32);

    // Display updated frame
    epd.display_frame(&mut spi, &mut delay).unwrap();
    info!("drew squares");
    delay.delay_ms(50000_u32);

    info!("sleeping display...");
    // Set the EPD to sleep
    epd.sleep(&mut spi, &mut delay).unwrap();

    loop {
        // info!("on!");
        // led_pin.set_high().unwrap();
        // delay.delay_ms(100);
        // info!("off!");
        // led_pin.set_low().unwrap();
        // delay.delay_ms(100);
    }
}
