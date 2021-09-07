use bitflags::bitflags;
use std::borrow::Cow;

use crate::sys;
use crate::Ui;

// TODO: support size constraints

/// Combo box height mode.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComboBoxHeight {
    /// Max ~4 items visible.
    Small,
    /// Max ~8 items visible.
    Regular,
    /// Max ~20 items visible.
    Large,
    /// As many fitting items as possible visible.
    Largest,
}

/// Combo box preview mode.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComboBoxPreviewMode {
    /// Show only a box with the preview value
    Label,
    /// Show only an arrow button
    ArrowButton,
    /// Show a box with the preview value and an arrow button
    Full,
}

bitflags!(
/// Flags for combo boxes
#[repr(transparent)]
pub struct ComboBoxFlags: u32 {
    /// Align the popup toward the left by default
    const POPUP_ALIGN_LEFT = sys::ImGuiComboFlags_PopupAlignLeft;
    /// Max ~4 items visible.
    const HEIGHT_SMALL = sys::ImGuiComboFlags_HeightSmall;
    /// Max ~8 items visible (default)
    const HEIGHT_REGULAR = sys::ImGuiComboFlags_HeightRegular;
    /// Max ~20 items visible
    const HEIGHT_LARGE = sys::ImGuiComboFlags_HeightLarge;
    /// As many fitting items as possible
    const HEIGHT_LARGEST = sys::ImGuiComboFlags_HeightLargest;
    /// Display on the preview box without the square arrow button
    const NO_ARROW_BUTTON = sys::ImGuiComboFlags_NoArrowButton;
    /// Display only a square arrow button
    const NO_PREVIEW = sys::ImGuiComboFlags_NoPreview;
}
);

/// Builder for a combo box widget
#[derive(Copy, Clone, Debug)]
#[must_use]
pub struct ComboBox<T, P = String> {
    label: T,
    preview_value: Option<P>,
    flags: ComboBoxFlags,
}

impl<'a, T: AsRef<str>, P: AsRef<str>> ComboBox<T, P> {
    /// Constructs a new combo box builder.
    #[doc(alias = "BeginCombo")]
    pub fn new(label: T) -> Self {
        ComboBox {
            label,
            preview_value: None,
            flags: ComboBoxFlags::empty(),
        }
    }

    /// Sets the preview value displayed in the preview box (if visible).
    #[inline]
    pub fn preview_value(mut self, preview_value: P) -> Self {
        self.preview_value = Some(preview_value);
        self
    }

    /// Replaces all current settings with the given flags.
    #[inline]
    pub fn flags(mut self, flags: ComboBoxFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Enables/disables aligning the combo box popup toward the left.
    ///
    /// Disabled by default.
    #[inline]
    pub fn popup_align_left(mut self, popup_align_left: bool) -> Self {
        self.flags
            .set(ComboBoxFlags::POPUP_ALIGN_LEFT, popup_align_left);
        self
    }

    /// Sets the combo box height.
    ///
    /// Default: `ComboBoxHeight::Regular`
    #[inline]
    pub fn height(mut self, height: ComboBoxHeight) -> Self {
        self.flags
            .set(ComboBoxFlags::HEIGHT_SMALL, height == ComboBoxHeight::Small);
        self.flags.set(
            ComboBoxFlags::HEIGHT_REGULAR,
            height == ComboBoxHeight::Regular,
        );
        self.flags
            .set(ComboBoxFlags::HEIGHT_LARGE, height == ComboBoxHeight::Large);
        self.flags.set(
            ComboBoxFlags::HEIGHT_LARGEST,
            height == ComboBoxHeight::Largest,
        );
        self
    }

    /// Sets the combo box preview mode.
    ///
    /// Default: `ComboBoxPreviewMode::Full`
    #[inline]
    pub fn preview_mode(mut self, preview_mode: ComboBoxPreviewMode) -> Self {
        self.flags.set(
            ComboBoxFlags::NO_ARROW_BUTTON,
            preview_mode == ComboBoxPreviewMode::Label,
        );
        self.flags.set(
            ComboBoxFlags::NO_PREVIEW,
            preview_mode == ComboBoxPreviewMode::ArrowButton,
        );
        self
    }

    /// Creates a combo box and starts appending to it.
    ///
    /// Returns `Some(ComboBoxToken)` if the combo box is open. After content has been
    /// rendered, the token must be ended by calling `.end()`.
    ///
    /// Returns `None` if the combo box is not open and no content should be rendered.
    #[must_use]
    pub fn begin<'ui>(self, ui: &Ui<'ui>) -> Option<ComboBoxToken<'ui>> {
        let should_render = unsafe {
            if let Some(preview_value) = self.preview_value {
                let (ptr_one, ptr_two) = ui.scratch_txt_two(self.label, preview_value);
                sys::igBeginCombo(ptr_one, ptr_two, self.flags.bits() as i32)
            } else {
                let ptr_one = ui.scratch_txt(self.label);
                sys::igBeginCombo(ptr_one, std::ptr::null(), self.flags.bits() as i32)
            }
        };
        if should_render {
            Some(ComboBoxToken::new(ui))
        } else {
            None
        }
    }
    /// Creates a combo box and runs a closure to construct the popup contents.
    /// Returns the result of the closure, if it is called.
    ///
    /// Note: the closure is not called if the combo box is not open.
    pub fn build<R, F: FnOnce() -> R>(self, ui: &Ui, f: F) -> Option<R> {
        self.begin(ui).map(|_combo| f())
    }
}

create_token!(
    /// Tracks a combo box that can be ended by calling `.end()`
    /// or by dropping.
    pub struct ComboBoxToken<'ui>;

    /// Ends a combo box
    drop { sys::igEndCombo() }
);

/// # Convenience functions
impl<'a, T: AsRef<str> + Clone> ComboBox<T, Cow<'a, str>> {
    /// Builds a simple combo box for choosing from a slice of values
    #[doc(alias = "BeginCombo")]
    pub fn build_simple<V, L>(
        mut self,
        ui: &Ui,
        current_item: &mut usize,
        items: &'a [V],
        label_fn: &L,
    ) -> bool
    where
        for<'b> L: Fn(&'b V) -> Cow<'b, str>,
    {
        use crate::widget::selectable::Selectable;
        let mut result = false;
        let preview_value = items.get(*current_item).map(label_fn);
        if self.preview_value.is_none() {
            if let Some(preview_value) = preview_value {
                self = self.preview_value(preview_value);
            }
        }
        if let Some(_cb) = self.begin(ui) {
            for (idx, item) in items.iter().enumerate() {
                let text = label_fn(item);
                let selected = idx == *current_item;
                if Selectable::new(&text).selected(selected).build(ui) {
                    *current_item = idx;
                    result = true;
                }
                if selected {
                    ui.set_item_default_focus();
                }
            }
        }
        result
    }
    /// Builds a simple combo box for choosing from a slice of strings
    #[doc(alias = "BeginCombo")]
    pub fn build_simple_string<S>(self, ui: &Ui, current_item: &mut usize, items: &[&S]) -> bool
    where
        S: AsRef<str> + ?Sized,
    {
        self.build_simple(ui, current_item, items, &|&s| s.as_ref().into())
    }
}
