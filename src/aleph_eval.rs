//! ℵ-OS λ_ℵ evaluator — resolves AST nodes into Letter tuples.
//!
//! Carries a session environment (let bindings) and evaluates expressions
//! using the core lattice operations from `crate::aleph`.

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::format;
use alloc::vec::Vec;

use crate::aleph;
use crate::aleph::Tuple;
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
    Letter(&'static aleph::LetterDef),  // A resolved letter
    Unit,                                // void (census, let binding side-effect)
    Distance(f64, Vec<usize>),           // (distance, conflict_set indices)
}

impl EvalResult {
    /// If this is a Letter, return its tuple.
    pub fn tuple(&self) -> Option<Tuple> {
        match self {
            EvalResult::Letter(l) => Some(l.t),
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
                // Try as letter name
                aleph::letter_by_name(name)
                    .map(EvalResult::Letter)
                    .ok_or_else(|| format!("Unknown letter: '{}'", name))
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
                // Note: probe output is printed by the caller
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

            Expr::Let(name, expr) => {
                let result = self.eval(expr)?;
                if let EvalResult::Letter(l) = result {
                    self.env.insert(name.clone(), l);
                    Ok(EvalResult::Letter(l))
                } else {
                    Err(format!("let binding requires a letter, got {:?}", result))
                }
            }

            Expr::Palace(n, inner) => {
                // Palace(n) asserts that the inner expression's tier >= the
                // palace barrier threshold. If the barrier passes, returns
                // the inner result unchanged.
                //
                // Barrier mapping (from Python aleph_1.py):
                //   palace 1,2 -> requires O_0  (always passes)
                //   palace 3   -> requires O_1
                //   palace 4,5,6 -> requires O_2
                //   palace 7+  -> requires O_inf
                let result = self.eval(inner)?;
                if let EvalResult::Letter(l) = &result {
                    let tier = aleph::compute_tier(&l.t);
                    let required_tier = match n {
                        1 | 2 => aleph::Tier::O0,
                        3 => aleph::Tier::O1,
                        4 | 5 | 6 => aleph::Tier::O2,
                        _ => aleph::Tier::OInf,  // 7+
                    };
                    // Check barrier: required tier must be <= actual tier
                    // Tier ordering: O0 < O1 < O2 < O2d < OInf
                    let tier_ok = match (required_tier, tier) {
                        (aleph::Tier::O0, _) => true,          // O_0 always passes
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
                }
                Ok(result)
            }

            Expr::Match(scrutinee, arms) => {
                // Evaluate the scrutinee to get its tier, then dispatch to
                // the matching arm.
                let t = self.eval_expr_tuple(scrutinee)?;
                let tier = aleph::compute_tier(&t);
                let tier_name = aleph::tier_name(tier);
                // Find the matching arm
                for arm in arms {
                    let matches = match (&arm.pattern, tier) {
                        (_, _) if matches_pattern(&arm.pattern, tier_name) => true,
                        _ => false,
                    };
                    if matches {
                        return self.eval(&arm.expr);
                    }
                }
                Err(format!("match: no arm matched tier '{}'", tier_name))
            }
        }
    }

    /// Evaluate an expression node and return its tuple directly.
    fn eval_expr_tuple(&mut self, expr: &Expr) -> Result<Tuple, String> {
        match self.eval(expr)? {
            EvalResult::Letter(l) => Ok(l.t),
            EvalResult::Unit => Err("Expected a letter, got void".into()),
            EvalResult::Distance(_, _) => Err("Expected a letter, got distance".into()),
        }
    }

    /// Find the closest canonical letter to a given tuple.
    /// Returns the best match, or a synthetic "anonymous" letter if none is close.
    fn find_best_letter(t: &Tuple) -> &'static aleph::LetterDef {
        let mut best_dist = u32::MAX;
        let mut best_idx = 0;
        for (i, l) in aleph::LETTERS.iter().enumerate() {
            let d = aleph::distance_scaled(&l.t, t);
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }
        &aleph::LETTERS[best_idx]
    }
}

// ── Probe output helpers (called by REPL after eval) ─────────────────────────

/// Format probe_Φ output.
pub fn format_probe_phi(t: &Tuple) -> String {
    let phi_val = t[8] as usize;
    let phi_name = aleph::PHI_NAMES.get(phi_val).copied().unwrap_or("?");
    format!("  probe_Phi -> {}  (ordinal {})", phi_name, t[8])
}

/// Format probe_Ω output.
pub fn format_probe_omega(t: &Tuple) -> String {
    let om_val = t[11] as usize;
    let om_name = aleph::OMEGA_NAMES.get(om_val).copied().unwrap_or("?");
    format!("  probe_Omega -> {}  (ordinal {})", om_name, t[11])
}

/// Format tier output.
pub fn format_tier(t: &Tuple) -> String {
    let tier = aleph::compute_tier(t);
    format!("  tier -> {}", aleph::tier_name(tier))
}

/// Format census output.
pub fn format_census() -> String {
    use alloc::format;
    let counts = aleph::tier_census();
    let tier_labels = ["O_0", "O_1", "O_2", "O_2d", "O_inf"];
    let mut s = String::new();
    for i in 0..5 {
        if counts[i] > 0 {
            s += &format!("  {} ({:2}): ", tier_labels[i], counts[i]);
            // List letters in this tier
            let mut first = true;
            for l in &aleph::LETTERS {
                let tier_idx = match aleph::compute_tier(&l.t) {
                    aleph::Tier::O0 => 0,
                    aleph::Tier::O1 => 1,
                    aleph::Tier::O2 => 2,
                    aleph::Tier::O2d => 3,
                    aleph::Tier::OInf => 4,
                };
                if tier_idx == i {
                    if !first { s += ", "; }
                    s += l.name;
                    first = false;
                }
            }
            s += "\n";
        }
    }
    s
}
