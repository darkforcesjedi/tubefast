use crate::app::Action;
use crate::ytm::Item;

#[cfg(windows)]
pub struct Media {
    controls: windows::Media::SystemMediaTransportControls,
    track: String,
    playing: bool,
}

#[cfg(windows)]
impl Media {
    pub fn new(cc: &eframe::CreationContext<'_>, press: impl Fn(Action) + Send + 'static) -> Option<Self> {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use windows::Foundation::TypedEventHandler;
        use windows::Media::{
            SystemMediaTransportControls, SystemMediaTransportControlsButton as Button,
            SystemMediaTransportControlsButtonPressedEventArgs as Pressed,
        };
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::WinRT::ISystemMediaTransportControlsInterop as Interop;
        use windows::core::{Ref, factory};

        let RawWindowHandle::Win32(window) = cc.window_handle().ok()?.as_raw() else {
            return None;
        };
        let interop = factory::<SystemMediaTransportControls, Interop>().ok()?;
        let controls: SystemMediaTransportControls = unsafe { interop.GetForWindow(HWND(window.hwnd.get() as *mut _)) }.ok()?;
        controls.SetIsEnabled(true).ok()?;
        controls.SetIsPlayEnabled(true).ok()?;
        controls.SetIsPauseEnabled(true).ok()?;
        controls.SetIsNextEnabled(true).ok()?;
        controls.SetIsPreviousEnabled(true).ok()?;
        let pressed = TypedEventHandler::new(move |_, pressed: Ref<Pressed>| {
            let button = pressed.ok()?.Button()?;
            if button == Button::Play {
                press(Action::Pause(false));
            } else if button == Button::Pause || button == Button::Stop {
                press(Action::Pause(true));
            } else if button == Button::Next {
                press(Action::Next);
            } else if button == Button::Previous {
                press(Action::Previous);
            }
            Ok(())
        });
        controls.ButtonPressed(&pressed).ok()?;
        Some(Self {
            controls,
            track: String::new(),
            playing: false,
        })
    }

    pub fn show(&mut self, item: Option<&Item>, playing: bool) {
        let track = item.map_or("", |item| item.video_id.as_str());
        if self.track == track && self.playing == playing {
            return;
        }
        (self.track, self.playing) = (track.to_owned(), playing);
        let _ = self.publish(item, playing);
    }

    fn publish(&self, item: Option<&Item>, playing: bool) -> windows::core::Result<()> {
        use windows::Media::{MediaPlaybackStatus as Status, MediaPlaybackType};
        use windows::core::HSTRING;

        let Some(item) = item else {
            return self.controls.SetPlaybackStatus(Status::Closed);
        };
        self.controls
            .SetPlaybackStatus(if playing { Status::Playing } else { Status::Paused })?;
        let display = self.controls.DisplayUpdater()?;
        display.SetType(MediaPlaybackType::Music)?;
        let music = display.MusicProperties()?;
        music.SetTitle(&HSTRING::from(item.title.as_str()))?;
        music.SetArtist(&HSTRING::from(item.subtitle.as_str()))?;
        display.Update()
    }
}

#[cfg(not(windows))]
pub struct Media;

#[cfg(not(windows))]
impl Media {
    pub fn new(_cc: &eframe::CreationContext<'_>, _press: impl Fn(Action) + Send + 'static) -> Option<Self> {
        None
    }

    pub fn show(&mut self, _item: Option<&Item>, _playing: bool) {}
}
