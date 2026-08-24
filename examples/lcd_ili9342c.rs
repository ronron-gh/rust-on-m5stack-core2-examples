// esp-idf-halのexamplesのspi_st7789.rsをベースに、以下情報を参考にしてカスタマイズ。
// 「M5StackをRustで動かす」(https://zenn.dev/teruyamato0731/scraps/eaf1afddd92124)

use embedded_hal::spi::MODE_3;

//use esp_idf_hal::delay::Ets;      // 代わりにFreeRtosを使用。
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_hal::units::FromValueType;
use esp_idf_hal::units::*;
use esp_idf_hal::i2c::*;

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::image::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::{Text, Alignment};
use embedded_graphics::mono_font::{ascii::FONT_7X13, MonoTextStyle};

use mipidsi::{Builder, Orientation};

use axp192::Axp192;

fn main() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;

    let rst = PinDriver::output(peripherals.pins.gpio4)?;
    let dc = PinDriver::output(peripherals.pins.gpio15)?;
    let mut backlight = PinDriver::output(peripherals.pins.gpio3)?;
    let sclk = peripherals.pins.gpio18;
    let sda = peripherals.pins.gpio23;
    let sdi = peripherals.pins.gpio38;
    let cs = peripherals.pins.gpio5;

    //let mut delay = Ets;
    let mut delay = FreeRtos;

    // axp192の初期化、LCDに電源投入
    let mut i2c_master = i2c_master_init(
        peripherals.i2c0,
        peripherals.pins.gpio21.into(),
        peripherals.pins.gpio22.into(),
        400.kHz().into(),
    )?;

    let mut axp = Axp192::new(i2c_master);
    m5sc2_init(&mut axp, &mut delay).unwrap();


    // configuring the spi interface, note that in order for the ST7789 to work, the data_mode needs to be set to MODE_3
    let config = config::Config::new()
        .baudrate(26.MHz().into())
        .data_mode(MODE_3);

    let device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sda,
        Some(sdi),
        Some(cs),
        &SpiDriverConfig::new(),
        &config,
    )?;

    // display interface abstraction from SPI and DC
    let di = SPIInterfaceNoCS::new(device, dc);

    // create driver
    let mut display = Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_color_order(mipidsi::ColorOrder::Bgr)
        .with_invert_colors(true)
        .with_orientation(Orientation::Portrait(false))
        // initialize
        .init(&mut delay, Some(rst))
        .unwrap();

    // turn on the backlight
    backlight.set_high()?;
    let raw_image_data = ImageRawLE::new(include_bytes!("assets/ferris.raw"), 86);
    let ferris = Image::new(&raw_image_data, Point::new(20, 150));

    // draw image on black background
    display.clear(Rgb565::BLACK).unwrap();
    ferris.draw(&mut display).unwrap();

    // draw a green line
    Line::new(Point::new(50, 20), Point::new(270, 220))
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::GREEN, 1))
        .draw(&mut display)
        .unwrap();

    // draw a red circle
    Circle::new(Point::new(260, 30), 30)    // 中心、半径
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 5))
        .draw(&mut display)
        .unwrap();

    // draw a blue triangle
    Triangle::new(Point::new(200, 20), Point::new(170, 45), Point::new(300, 150))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))   // 塗りつぶし
        .draw(&mut display)
        .unwrap();

    // シアンの太線で囲われていて、黄色で塗りつぶされた長方形
    let style = PrimitiveStyleBuilder::new()
        .stroke_width(10)
        .stroke_color(Rgb565::CYAN)
        .fill_color(Rgb565::YELLOW)
        .build();
    
    Rectangle::new(Point::new(100, 100), Size::new(120, 40))    // 左上頂点、サイズ
        .into_styled(style)
        .draw(&mut display)
        .unwrap();
    
    // draw text
    Text::with_alignment(
        "hello world!",
        Point::new(160, 120),
        MonoTextStyle::new(&FONT_7X13, Rgb565::BLACK),
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    println!("Image printed!");

    loop {
        FreeRtos::delay_ms(1000);
    }
}

// esp-idf-halのexamplesのi2c_master_slave.rsから持ってきた
fn i2c_master_init<'d>(
    i2c: impl I2c + 'd,
    sda: AnyIOPin<'d>,
    scl: AnyIOPin<'d>,
    baudrate: Hertz,
) -> anyhow::Result<I2cDriver<'d>> {
    let config = I2cConfig::new().baudrate(baudrate);
    let driver = I2cDriver::new(i2c, sda, scl, &config)?;
    Ok(driver)
}

// axp192-rsのexampleから持ってきた
fn m5sc2_init<I2C, E>(axp: &mut axp192::Axp192<I2C>, _delay: &mut FreeRtos) -> Result<(), E>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    // Default setup for M5Stack Core 2
    axp.set_dcdc1_voltage(3350)?; // Voltage to provide to the microcontroller (this one!)

    axp.set_ldo2_voltage(3300)?; // Peripherals (LCD, ...)
    axp.set_ldo2_on(true)?;

    axp.set_ldo3_voltage(2000)?; // Vibration motor
    axp.set_ldo3_on(false)?;

    axp.set_dcdc3_voltage(2800)?; // LCD backlight
    axp.set_dcdc3_on(true)?;

    axp.set_gpio1_mode(axp192::GpioMode12::NmosOpenDrainOutput)?; // Power LED
    axp.set_gpio1_output(false)?; // In open drain modes, state is opposite to what you might
                                  // expect

    axp.set_gpio2_mode(axp192::GpioMode12::NmosOpenDrainOutput)?; // Speaker
    axp.set_gpio2_output(true)?;

    axp.set_key_mode(
        // Configure how the power button press will work
        axp192::ShutdownDuration::Sd4s,
        axp192::PowerOkDelay::Delay64ms,
        true,
        axp192::LongPress::Lp1000ms,
        axp192::BootTime::Boot512ms,
    )?;

    axp.set_gpio4_mode(axp192::GpioMode34::NmosOpenDrainOutput)?; // LCD reset control

    axp.set_battery_voltage_adc_enable(true)?;
    axp.set_battery_current_adc_enable(true)?;
    axp.set_acin_current_adc_enable(true)?;
    axp.set_acin_voltage_adc_enable(true)?;

    // Actually reset the LCD
    axp.set_gpio4_output(false)?;
    axp.set_ldo3_on(true)?; // Buzz the vibration motor while intializing ¯\_(ツ)_/¯
    //delay.delay_millis(100u32);
    FreeRtos::delay_ms(100u32);     //_delay.delay_ms()ではダメだった
    axp.set_gpio4_output(true)?;
    axp.set_ldo3_on(false)?;
    //delay.delay_millis(100u32);
    FreeRtos::delay_ms(100u32);
    Ok(())
}