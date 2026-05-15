//! Streaming markdown → ANSI renderer.
//!
//! Feeds one char at a time. Maintains a small state machine for `**bold**`,
//! `*italic*` / `_italic_`, `` `code` `` / ` ```fenced``` `, and `# heading`.
//! Word-wraps at `max_w` columns to keep output readable in narrow terminals.

use std::io::{self, Write};

pub struct Renderer {
    max_w: usize,
    col: usize,
    pending: String,
    last_char: char,

    md_bold: bool,
    md_italic: bool,
    md_code: bool,
    md_fenced: bool,
    md_heading: bool,
    md_marker_buf: String,
    md_at_line_start: bool,
    md_hash_pending: String,

    emitted_bold: bool,
    emitted_dim: bool,
}

impl Renderer {
    pub fn new(max_w: usize) -> Self {
        Self {
            max_w,
            col: 0,
            pending: String::new(),
            last_char: '\0',
            md_bold: false,
            md_italic: false,
            md_code: false,
            md_fenced: false,
            md_heading: false,
            md_marker_buf: String::new(),
            md_at_line_start: true,
            md_hash_pending: String::new(),
            emitted_bold: false,
            emitted_dim: false,
        }
    }

    pub fn feed(&mut self, ch: char) {
        if !self.md_hash_pending.is_empty() {
            if ch == '#' {
                self.md_hash_pending.push('#');
                return;
            }
            if ch == ' ' {
                self.set_heading(true);
                self.md_hash_pending.clear();
                return;
            }
            let hp = std::mem::take(&mut self.md_hash_pending);
            for c in hp.chars() {
                self.emit(c);
            }
            self.md_at_line_start = false;
        }

        if ch == '\n' {
            if !self.md_marker_buf.is_empty() {
                self.classify_markers(Some('\n'));
            }
            if self.md_heading {
                self.set_heading(false);
            }
            self.emit('\n');
            self.md_at_line_start = true;
            return;
        }

        if self.md_fenced {
            if ch == '`' {
                self.md_marker_buf.push(ch);
                if self.md_marker_buf.len() >= 3 && self.md_marker_buf.ends_with("```") {
                    self.set_fenced(false);
                    self.md_marker_buf.clear();
                }
            } else {
                if !self.md_marker_buf.is_empty() {
                    let mb = std::mem::take(&mut self.md_marker_buf);
                    for c in mb.chars() {
                        self.emit(c);
                    }
                }
                self.emit(ch);
            }
            return;
        }

        if self.md_at_line_start && ch == '#' {
            self.md_hash_pending.push('#');
            self.md_at_line_start = false;
            return;
        }

        self.md_at_line_start = false;

        if matches!(ch, '*' | '`' | '_') {
            self.md_marker_buf.push(ch);
            return;
        }

        if !self.md_marker_buf.is_empty() {
            self.classify_markers(Some(ch));
        }

        self.emit(ch);
    }

    pub fn finish(&mut self) {
        if !self.md_marker_buf.is_empty() {
            self.classify_markers(None);
        }
        self.flush_pending();
        self.write("\x1b[0m");
        let _ = io::stdout().flush();
    }

    fn write(&self, s: &str) {
        let _ = io::stdout().write_all(s.as_bytes());
    }

    fn emit(&mut self, ch: char) {
        match ch {
            '\n' => {
                if !self.pending.is_empty() {
                    let plen = self.pending.chars().count();
                    if self.col + plen > self.max_w {
                        self.write("\n");
                        self.col = 0;
                    }
                    let p = std::mem::take(&mut self.pending);
                    self.write(&p);
                }
                self.write("\n");
                self.col = 0;
            }
            ' ' => {
                if !self.pending.is_empty() {
                    let plen = self.pending.chars().count();
                    if self.col + plen > self.max_w {
                        self.write("\n");
                        self.col = 0;
                    }
                    let p = std::mem::take(&mut self.pending);
                    self.write(&p);
                    self.col += plen;
                }
                if self.col + 1 > self.max_w {
                    self.write("\n");
                    self.col = 0;
                } else {
                    self.write(" ");
                    self.col += 1;
                }
            }
            _ => {
                self.pending.push(ch);
            }
        }
        self.last_char = ch;
    }

    fn flush_pending(&mut self) {
        if !self.pending.is_empty() {
            let plen = self.pending.chars().count();
            if self.col + plen > self.max_w {
                self.write("\n");
                self.col = 0;
            }
            let p = std::mem::take(&mut self.pending);
            self.write(&p);
            self.col += plen;
        }
        let _ = io::stdout().flush();
    }

    fn sync_attrs(&mut self) {
        let want_bold = self.md_bold || self.md_heading;
        let want_dim = self.md_fenced;
        if (!want_bold && self.emitted_bold) || (!want_dim && self.emitted_dim) {
            self.write("\x1b[22m");
            self.emitted_bold = false;
            self.emitted_dim = false;
        }
        if want_bold && !self.emitted_bold {
            self.write("\x1b[1m");
            self.emitted_bold = true;
        }
        if want_dim && !self.emitted_dim {
            self.write("\x1b[2m");
            self.emitted_dim = true;
        }
    }

    fn set_bold(&mut self, on: bool) {
        if self.md_bold == on {
            return;
        }
        self.flush_pending();
        self.md_bold = on;
        self.sync_attrs();
    }

    fn set_heading(&mut self, on: bool) {
        if self.md_heading == on {
            return;
        }
        self.flush_pending();
        self.md_heading = on;
        self.sync_attrs();
    }

    fn set_fenced(&mut self, on: bool) {
        if self.md_fenced == on {
            return;
        }
        self.flush_pending();
        self.md_fenced = on;
        self.sync_attrs();
    }

    fn set_code(&mut self, on: bool) {
        if self.md_code == on {
            return;
        }
        self.flush_pending();
        self.md_code = on;
        self.write(if on { "\x1b[36m" } else { "\x1b[39m" });
    }

    fn set_italic(&mut self, on: bool) {
        if self.md_italic == on {
            return;
        }
        self.flush_pending();
        self.md_italic = on;
        self.write(if on { "\x1b[3m" } else { "\x1b[23m" });
    }

    fn classify_markers(&mut self, next_ch: Option<char>) {
        let mut s = std::mem::take(&mut self.md_marker_buf);
        let last_char = self.last_char;
        while !s.is_empty() {
            if s.starts_with("```") {
                let on = !self.md_fenced;
                self.set_fenced(on);
                s.drain(..3);
            } else if s.starts_with("**") {
                let on = !self.md_bold;
                self.set_bold(on);
                s.drain(..2);
            } else if s.starts_with('`') {
                let on = !self.md_code;
                self.set_code(on);
                s.drain(..1);
            } else if s.starts_with('*') {
                let opening = !self.md_italic;
                if opening {
                    let allow = next_ch.map(|c| c != ' ' && c != '\n').unwrap_or(false);
                    if allow {
                        self.set_italic(true);
                    } else {
                        self.emit('*');
                    }
                } else {
                    let allow = last_char != '\0' && last_char != ' ' && last_char != '\n';
                    if allow {
                        self.set_italic(false);
                    } else {
                        self.emit('*');
                    }
                }
                s.drain(..1);
            } else if s.starts_with('_') {
                let opening = !self.md_italic;
                if opening {
                    let lc_ok = last_char == '\0' || !last_char.is_alphanumeric();
                    let nc_ok = next_ch.map(|c| c != ' ' && c != '\n').unwrap_or(false);
                    if lc_ok && nc_ok {
                        self.set_italic(true);
                    } else {
                        self.emit('_');
                    }
                } else {
                    let lc_ok = last_char != '\0' && last_char != ' ' && last_char != '\n';
                    let nc_ok = next_ch.map(|c| !c.is_alphanumeric()).unwrap_or(true);
                    if lc_ok && nc_ok {
                        self.set_italic(false);
                    } else {
                        self.emit('_');
                    }
                }
                s.drain(..1);
            } else {
                let first = s.chars().next().unwrap();
                self.emit(first);
                let n = first.len_utf8();
                s.drain(..n);
            }
        }
    }
}
