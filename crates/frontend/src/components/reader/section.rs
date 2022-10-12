use wasm_bindgen::prelude::Closure;
use web_sys::HtmlIFrameElement;

use super::CachedPage;

pub enum SectionLoadProgress {
    Waiting,
    Loading(SectionContents),
    Loaded(SectionContents),
}

impl SectionLoadProgress {
    pub fn is_waiting(&self) -> bool {
        matches!(self, Self::Waiting)
    }

    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading(_))
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded(_))
    }

    pub fn convert_to_loaded(&mut self) {
        if let Self::Loading(v) = std::mem::replace(self, Self::Waiting) {
            *self = Self::Loaded(v);
        } else {
            panic!("unable to convert")
        }
    }

    pub fn as_loaded(&self) -> Option<&SectionContents> {
        match self {
            Self::Loaded(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_chapter(&self) -> Option<&SectionContents> {
        match self {
            Self::Loading(v) | Self::Loaded(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_chapter_mut(&mut self) -> Option<&mut SectionContents> {
        match self {
            Self::Loading(v) | Self::Loaded(v) => Some(v),
            _ => None,
        }
    }
}

pub struct SectionContents {
    #[allow(dead_code)]
    on_load: Closure<dyn FnMut()>,

    cached_pages: Vec<CachedPage>,

    iframe: HtmlIFrameElement,
    chapter: usize,

    /// Global Page Index
    pub gpi: usize,

    pub viewing_page: usize,
}

impl SectionContents {
    pub fn new(chapter: usize, iframe: HtmlIFrameElement, on_load: Closure<dyn FnMut()>) -> Self {
        Self {
            on_load,
            cached_pages: Vec::new(),
            iframe,
            chapter,
            gpi: 0,
            viewing_page: 0,
        }
    }

    pub fn get_iframe(&self) -> &HtmlIFrameElement {
        &self.iframe
    }

    pub fn set_cached_pages(&mut self, value: Vec<CachedPage>) {
        self.cached_pages = value;
    }

    pub fn page_count(&self) -> usize {
        self.cached_pages.len()
    }

    pub fn viewing_page(&self) -> usize {
        self.viewing_page
    }

    pub fn chapter(&self) -> usize {
        self.chapter
    }

    pub fn get_page_count_until(&self) -> usize {
        self.gpi + self.page_count()
    }

    pub fn transitioning_page(&self, amount: isize) {
        let body = self.iframe.content_document().unwrap().body().unwrap();

        // Prevent empty pages when on the first or last page of a section.
        let amount = if (amount.is_positive() && self.viewing_page == 0)
            || (amount.is_negative() && self.viewing_page == self.page_count().saturating_sub(1))
        {
            0
        } else {
            amount
        };

        if amount == 0 {
            body.style()
                .set_property("transition", "left 0.5s ease 0s")
                .unwrap();
        } else {
            body.style().remove_property("transition").unwrap();
        }

        body.style()
            .set_property(
                "left",
                &format!(
                    "calc(-{}% - {}px)",
                    100 * self.viewing_page,
                    self.viewing_page as isize * 10 - amount
                ),
            )
            .unwrap();
    }
}

impl PartialEq for SectionContents {
    fn eq(&self, other: &Self) -> bool {
        self.chapter == other.chapter
    }
}
