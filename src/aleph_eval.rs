//! ℵ-OS λ_ℵ evaluator — resolves AST nodes into Letter tuples.
//!
//! Carries a session environment (let bindings) and evaluates expressions
//! using the core lattice operations from `crate::aleph`.
//!
//! Extended with native 14-Sefirot support: Sefirah names resolve to their
//! imscribed 12-primitive tuples via `crate::aleph_sefirot`.

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::format;
use alloc::vec::Vec;

use crate::aleph;
use crate::aleph::Tuple;
use crate::aleph_sefirot;
use crate::aleph_parser::{Expr, TierPattern};

/// Check if a tier pattern matches a given tier name.
fn matches_pattern(pattern: &TierPattern, tier_name: &str) -> bool {
    match pattern {
        TierPattern::O0 => tier_name == "O_0",
        TierPattern::O1 => tier_name == "O_1",
        TierPattern::O2 => tier_name == "O_2",
        TierPattern::O2d => tier_name == "O_2d",
        TierPattern::OInf => tier_name == "O_inf",
        TierPattern::Wildcard => true,
    }
}

// ── Evaluation result ────────────────────────────────────────────────────────

/// What the evaluator can return.
#[derive(Debug, Clone)]
pub enum EvalResult {
    Letter(&'static aleph::LetterDef),   // A resolved letter
    Sefirah(&'static aleph_sefirot::SefirahDef), // A resolved Sefirah
    Unit,                                 // void (census, let binding side-effect)
    Distance(f64, Vec<usize>),            // (distance, conflict_set indices)
    Emanation(Vec<aleph_sefirot::EmanationStep>), // emanation chain
    SefirahCensus([u8; 5]),              // 14-Sefirot tier census
}

impl EvalResult {
    /// If this is a Letter, return its tuple.
    pub fn tuple(&self) -> Option<Tuple> {
        match self {
            EvalResult::Letter(l) => Some(l.t),
            EvalResult::Sefirah(s) => Some(s.t),
            _ => None,
        }
    }

    /// If this is a Letter, return the LetterDef.
    pub fn letter(&self) -> Option<&'static aleph::LetterDef> {
        match self {
            EvalResult::Letter(l) => Some(l),
            _ => None,
        }
    }

    /// If this is a Sefirah, return the SefirahDef.
    pub fn sefirah(&self) -> Option<&'static aleph_sefirot::SefirahDef> {
        match self {
            EvalResult::Sefirah(s) => Some(s),
            _ => None,
        }
    }
}

// ── Session environment ──────────────────────────────────────────────────────

/// Named bindings: "x" -> LetterDef reference.
pub type Env = BTreeMap<String, &'static aleph::LetterDef>;

// ── Evaluator ─────────────────────────────────────────────────────────────────

pub struct Evaluator {
    env: Env,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator { env: BTreeMap::new() }
    }

    /// Bind a tuple to a name in the session environment.
    pub fn bind(&mut self, name: &str, t: aleph::Tuple) {
        // Find the best-matching letter for display purposes
        let l = Self::find_best_letter(&t);
        // Leak the letter to get a 'static reference
        // This is acceptable for a REPL session
        let leaked = Box::leak(Box::new(aleph::LetterDef {
            name: l.name,
            glyph: l.glyph,
            t,
        }));
        self.env.insert(name.to_string(), leaked);
    }

    /// Get the current session environment (immutable).
    pub fn env(&self) -> &Env {
        &self.env
    }

    /// Evaluate an expression in the session environment.
    /// Returns (result, new_bindings) — bindings are accumulated in-place.
    pub fn eval(&mut self, expr: &Expr) -> Result<EvalResult, String> {
        match expr {
            Expr::Letter(glyph) => {
                // Hebrew glyph literal
                aleph::letter_by_glyph(glyph.chars().next().unwrap())
                    .map(EvalResult::Letter)
                    .ok_or_else(|| format!("Unknown glyph: '{}'", glyph))
            }

            Expr::Name(name) => {
                // Variable or letter name
                if let Some(l) = self.env.get(name) {
                    return Ok(EvalResult::Letter(l));
                }
                // Try case-insensitive lookup
                for (key, val) in &self.env {
                    if key.eq_ignore_ascii_case(name) {
                        return Ok(EvalResult::Letter(val));
                    }
                }
                // Try as Sefirah name (14-Sefirot type system)
                if let Some(s) = aleph_sefirot::resolve_sefirah(name) {
                    return Ok(EvalResult::Sefirah(s));
                }
                // Try as Hebrew letter name
                aleph::letter_by_name(name)
                    .map(EvalResult::Letter)
                    .ok_or_else(|| format!("Unknown name: '{}' (not a letter, Sefirah, or binding)", name))
            }

            Expr::Tensor(a, b) => {
                let ta = self.eval_expr_tuple(a)?;
                let tb = self.eval_expr_tuple(b)?;
                let result = aleph::tensor(&ta, &tb);
                // Find the best-matching letter for display
                let matched = Self::find_best_letter(&result);
                Ok(EvalResult::Letter(matched))
            }

            Expr::Join(a, b) => {
                let ta = self.eval_expr_tuple(a)?;
                let tb = self.eval_expr_tuple(b)?;
                let result = aleph::join(&ta, &tb);
                let matched = Self::find_best_letter(&result);
                Ok(EvalResult::Letter(matched))
            }

            Expr::Meet(a, b) => {
                let ta = self.eval_expr_tuple(a)?;
                let tb = self.eval_expr_tuple(b)?;
                let result = aleph::meet(&ta, &tb);
                let matched = Self::find_best_letter(&result);
                Ok(EvalResult::Letter(matched))
            }

            Expr::Cast(src, target_name) => {
                let _src_tuple = self.eval_expr_tuple(src)?;
                // In the kernel, vav-cast is a type assertion.
                // For now, we just return the target letter.
                aleph::letter_by_name(target_name)
                    .map(EvalResult::Letter)
                    .ok_or_else(|| format!("Unknown target: '{}'", target_name))
            }

            Expr::ProbePhi(inner) => {
                let t = self.eval_expr_tuple(inner)?;
                let _phi_val = t[8] as usize;
                let _phi_name = aleph::PHI_NAMES.get(_phi_val).copied().unwrap_or("?");
                Ok(EvalResult::Letter(Self::find_best_letter(&t)))
            }

            Expr::ProbeOmega(inner) => {
                let t = self.eval_expr_tuple(inner)?;
                let om_val = t[11] as usize;
                let _om_name = aleph::OMEGA_NAMES.get(om_val).copied().unwrap_or("?");
                Ok(EvalResult::Letter(Self::find_best_letter(&t)))
            }

            Expr::Tier(inner) => {
                let t = self.eval_expr_tuple(inner)?;
                let _tier = aleph::compute_tier(&t);
                Ok(EvalResult::Letter(Self::find_best_letter(&t)))
            }

            Expr::Distance(a, b) => {
                let ta = self.eval_expr_tuple(a)?;
                let tb = self.eval_expr_tuple(b)?;
                let d = aleph::distance(&ta, &tb);
                let cs = aleph::conflict_set(&ta, &tb);
                Ok(EvalResult::Distance(d, cs))
            }
            Expr::Mediate(w, a, b) => {
                let tw = self.eval_expr_tuple(w)?;
                let ta = self.eval_expr_tuple(a)?;
                let tb = self.eval_expr_tuple(b)?;
                let result = aleph::mediate(&tw, &ta, &tb);
                let matched = Self::find_best_letter(&result);
                Ok(EvalResult::Letter(matched))
            }

            Expr::System => {
                let t = aleph::system_language();
                let matched = Self::find_best_letter(&t);
                Ok(EvalResult::Letter(matched))
            }

            Expr::Census => {
                Ok(EvalResult::Unit)
            }

            // ── 14-Sefirot built-ins ──────────────────────────────────────────

            Expr::SefirotCensus => {
                let census = aleph_sefirot::sefirah_census();
                Ok(EvalResult::SefirahCensus(census))
            }

            Expr::Emanation => {
                let chain = aleph_sefirot::emanation_chain();
                Ok(EvalResult::Emanation(chain))
            }

            Expr::SefirotLadder => {
                Ok(EvalResult::Unit) // Printed by caller
            }

            Expr::Let(name, expr) => {
                let result = self.eval(expr)?;
                if let EvalResult::Letter(l) = result {
                    self.env.insert(name.clone(), l);
                    Ok(EvalResult::Letter(l))
                } else if let EvalResult::Sefirah(s) = result {
                    // Bind the Sefirah tuple as a synthetic letter
                    let leaked = Box::leak(Box::new(aleph::LetterDef {
                        name: s.name,
                        glyph: '\0', // non-glyph
                        t: s.t,
                    }));
                    self.env.insert(name.clone(), leaked);
                    Ok(EvalResult::Sefirah(s))
                } else {
                    Err(format!("let binding requires a letter or Sefirah, got {:?}", result))
                }
            }

            Expr::Palace(n, inner) => {
                let result = self.eval(inner)?;
                if let EvalResult::Letter(l) = &result {
                    let tier = aleph::compute_tier(&l.t);
                    let required_tier = match n {
                        1 | 2 => aleph::Tier::O0,
                        3 => aleph::Tier::O1,
                        4 | 5 | 6 => aleph::Tier::O2,
                        _ => aleph::Tier::OInf,
                    };
                    let tier_ok = match (required_tier, tier) {
                        (aleph::Tier::O0, _) => true,
                        (aleph::Tier::O1, aleph::Tier::O0) => false,
                        (aleph::Tier::O1, _) => true,
                        (aleph::Tier::O2, aleph::Tier::O0) => false,
                        (aleph::Tier::O2, aleph::Tier::O1) => false,
                        (aleph::Tier::O2, _) => true,
                        (aleph::Tier::OInf, aleph::Tier::OInf) => true,
                        (_, _) => false,
                    };
                    if !tier_ok {
                        return Err(format!(
                            "Palace {} barrier violation: requires >= {}, got {}",
                            n, aleph::tier_name(required_tier), aleph::tier_name(tier)
                        ));
                    }
                    return Ok(result);
                }
                if let EvalResult::Sefirah(s) = &result {
                    let tier = aleph::compute_tier(&s.t);
                    let required_tier = match n {
                        1 | 2 => aleph::Tier::O0,
                        3 => aleph::Tier::O1,
                        4 | 5 | 6 => aleph::Tier::O2,
                        _ => aleph::Tier::OInf,
                    };
                    let tier_ok = match (required_tier, tier) {
                        (aleph::Tier::O0, _) => true,
                        (aleph::Tier::O1, aleph::Tier::O0) => false,
                        (aleph::Tier::O1, _) => true,
                        (aleph::Tier::O2, aleph::Tier::O0) => false,
                        (aleph::Tier::O2, aleph::Tier::O1) => false,
                        (aleph::Tier::O2, _) => true,
                        (aleph::Tier::OInf, aleph::Tier::OInf) => true,
                        (_, _) => false,
                    };
                    if !tier_ok {
                        return Err(format!(
                            "Palace {} barrier violation: requires >= {}, got {}",
                            n, aleph::tier_name(required_tier), aleph::tier_name(tier)
                        ));
                    }
                    return Ok(result);
                }
                Err(format!("Palace requires a letter or Sefirah result"))
            }

            Expr::Match(scrutinee, arms) => {
                let result = self.eval(scrutinee)?;
                let tier_name = match &result {
                    EvalResult::Letter(l) => aleph::tier_name(aleph::compute_tier(&l.t)),
                    EvalResult::Sefirah(s) => aleph::tier_name(aleph::compute_tier(&s.t)),
                    _ => return Err("match requires a letter or Sefirah scrutinee".into()),
                };
                for arm in arms {
                    if matches_pattern(&arm.pattern, tier_name) {
                        return self.eval(&arm.expr);
                    }
                }
                Err(format!("No match arm for tier '{}'", tier_name))
            }
        }
    }

    /// Evaluate and extract tuple from an expression.
    fn eval_expr_tuple(&mut self, expr: &Expr) -> Result<Tuple, String> {
        match self.eval(expr)? {
            EvalResult::Letter(l) => Ok(l.t),
            EvalResult::Sefirah(s) => Ok(s.t),
            _ => Err("Expected a letter or Sefirah, got something else".into()),
        }
    }

    /// Find the best-matching 22-letter for a tuple.
    fn find_best_letter(t: &Tuple) -> &'static aleph::LetterDef {
        let mut best_dist = u32::MAX;
        let mut best_idx = 0usize;
        for (i, letter) in aleph::LETTERS.iter().enumerate() {
            let d = aleph::distance_scaled(&letter.t, t);
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }
        &aleph::LETTERS[best_idx]
    }
}

// ── Formatting helpers (exported for aleph_repl / aleph_commands) ─────────────

pub fn format_census() -> String {
    let c = aleph::tier_census();
    format!(
        "  22-Letter Tier Distribution:\n\
         ----------------------------------------\n\
         O_0:    {}\n\
         O_1:    {}\n\
         O_2:    {}\n\
         O_2d:   {}\n\
         O_inf:  {}\n",
        c[0], c[1], c[2], c[3], c[4]
    )
}

pub fn format_sefirah_census() -> String {
    aleph_sefirot::format_sefirah_census()
}

pub fn format_probe_phi(t: &Tuple) -> String {
    let phi_val = t[8] as usize;
    let phi_name = aleph::PHI_NAMES.get(phi_val).copied().unwrap_or("?");
    format!("  probe_Φ: {} (index {})\n", phi_name, phi_val)
}

pub fn format_probe_omega(t: &Tuple) -> String {
    let om_val = t[11] as usize;
    let om_name = aleph::OMEGA_NAMES.get(om_val).copied().unwrap_or("?");
    format!("  probe_Ω: {} (index {})\n", om_name, om_val)
}

pub fn format_tier(t: &Tuple) -> String {
    let tier = aleph::compute_tier(t);
    format!("  tier: {}\n", aleph::tier_name(tier))
}

/// Format a probe_Φ result on a Sefirah tuple.
pub fn format_sefirah_probe_phi(s: &aleph_sefirot::SefirahDef) -> String {
    let phi_val = s.t[8] as usize;
    let phi_name = aleph::PHI_NAMES.get(phi_val).copied().unwrap_or("?");
    format!(
        "  probe_Φ: {} (index {})  gate: {}\n",
        phi_name, phi_val, aleph_sefirot::phi_gate_name(s.phi_gate)
    )
}

pub fn format_sefirah_probe_omega(s: &aleph_sefirot::SefirahDef) -> String {
    let om_val = s.t[11] as usize;
    let om_name = aleph::OMEGA_NAMES.get(om_val).copied().unwrap_or("?");
    format!("  probe_Ω: {} (index {})\n", om_name, om_val)
}

pub fn format_sefirah_tier(s: &aleph_sefirot::SefirahDef) -> String {
    let tier = aleph::compute_tier(&s.t);
    format!(
        "  tier: {}  gate: {}\n",
        aleph::tier_name(tier),
        aleph_sefirot::phi_gate_name(s.phi_gate)
    )
}
