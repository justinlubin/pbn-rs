//! Types and traits for [Programming by Navigation](https://dl.acm.org/doi/10.1145/3729264)
//!
//! Programming by Navigation is an interactive program synthesis problem in
//! which a [`StepProvider`] provides a list of next [`Step`]s that a step
//! _decider_ can choose between.
//!
//! To solve the Programming by Navigation Synthesis Problem, these steps are
//! required to satisfy properties called Strong Soundness and Strong
//! Completeness, which roughly say that all provided steps can lead to a valid
//! solution and that all possibly-reachable valid solutions are reachable among
//! just the provided steps (respectively). Validity is an arbitrary notion
//! defined by a [`ValidityChecker`].
//!
//! # Usage
//!
//! The [`Controller`] struct can be used to conveniently manage a Programming
//! by Navigation interactive session. Its API (and implementation) is a good
//! starting point to see how all the components hook together.

/// A cooperative timer used for early cutoff when synthesizing
pub trait Timer {
    /// The possible reasons for early cutoff (e.g., out of time, out of memory)
    type EarlyCutoff: std::error::Error;

    /// A cooperative "tick" of the timer
    fn tick(&self) -> Result<(), Self::EarlyCutoff>;
}

/// The interface for steps (also defines the notion of expression)
///
/// Steps transform one expression into another and must satisfy the
/// *navigation relation* properties in Section 3.1 of
/// [Programming by Navigation](https://dl.acm.org/doi/10.1145/3729264).
pub trait Step {
    /// The notion of expressions to use for Programming by Navigation
    type Exp: Clone;

    /// Returns the result of applying a step to an expression (which may fail)
    fn apply(&self, e: &Self::Exp) -> Option<Self::Exp>;
}

/// The interface for validity checking
pub trait ValidityChecker {
    /// The notion of expressions to use for Programming by Navigation
    type Exp;

    /// Returns whether or not the expression is valid
    fn check(&self, e: &Self::Exp) -> bool;
}

/// The interface for step providers
///
/// To be a valid solution to the Programming By Navigation Synthesis Problem,
/// step providers must satisfy the Validity, Strong Completeness, and Strong
/// Soundness properties in Section 3.2 of
/// [Programming by Navigation](https://dl.acm.org/doi/10.1145/3729264).
pub trait StepProvider<T: Timer> {
    /// The notion of steps that the step provider provides
    type Step: Step;

    /// Returns a set of provided steps given a current working expression
    fn provide(
        &mut self,
        timer: &T,
        e: &<Self::Step as Step>::Exp,
    ) -> Result<Vec<Self::Step>, T::EarlyCutoff>;
}

/// A composition of other step providers (all provided steps are concatenated)
pub struct CompoundProvider<T: Timer, S: Step> {
    providers: Vec<Box<dyn StepProvider<T, Step = S>>>,
}

impl<T: Timer, S: Step> CompoundProvider<T, S> {
    /// Creates a new [`CompoundProvider`] from a list of existing providers
    pub fn new(providers: Vec<Box<dyn StepProvider<T, Step = S>>>) -> Self {
        Self { providers }
    }
}

impl<T: Timer, S: Step> StepProvider<T> for CompoundProvider<T, S> {
    type Step = S;

    fn provide(
        &mut self,
        timer: &T,
        e: &<Self::Step as Step>::Exp,
    ) -> Result<Vec<Self::Step>, T::EarlyCutoff> {
        let mut steps = vec![];
        for p in &mut self.providers {
            steps.extend(p.provide(timer, e)?);
        }
        Ok(steps)
    }
}

/// A provider that returns the first provided step set that is nonempty (or
/// an empty set if there is none)
pub struct FallbackProvider<T: Timer, S: Step> {
    providers: Vec<Box<dyn StepProvider<T, Step = S>>>,
}

impl<T: Timer, S: Step> FallbackProvider<T, S> {
    /// Creates a new [`FallbackProvider`] from a list of existing providers
    pub fn new(providers: Vec<Box<dyn StepProvider<T, Step = S>>>) -> Self {
        Self { providers }
    }
}

impl<T: Timer, S: Step> StepProvider<T> for FallbackProvider<T, S> {
    type Step = S;

    fn provide(
        &mut self,
        timer: &T,
        e: &<Self::Step as Step>::Exp,
    ) -> Result<Vec<Self::Step>, T::EarlyCutoff> {
        for p in &mut self.providers {
            let steps = p.provide(timer, e)?;
            if !steps.is_empty() {
                return Ok(steps);
            }
        }
        Ok(vec![])
    }
}

/// A Programming by Navigation "controller" that abstracts away the underlying
/// step provider and validity checker to manage a Programming by Navigation
/// interactive session
pub struct Controller<T: Timer, S: Step> {
    timer: T,
    provider: Box<dyn StepProvider<T, Step = S> + 'static>,
    checker: Box<dyn ValidityChecker<Exp = S::Exp> + 'static>,
    state: S::Exp,
    history: Option<Vec<S::Exp>>,
}

impl<T: Timer, S: Step> Controller<T, S> {
    /// Create a new controller (history can be saved to enable meta-level
    /// "undo" operations in the interactive process)
    pub fn new(
        timer: T,
        provider: impl StepProvider<T, Step = S> + 'static,
        checker: impl ValidityChecker<Exp = S::Exp> + 'static,
        start: S::Exp,
        save_history: bool,
    ) -> Self {
        Self {
            timer,
            provider: Box::new(provider),
            checker: Box::new(checker),
            state: start,
            history: if save_history { Some(vec![]) } else { None },
        }
    }

    /// Ask the synthesizer to provide a list of possible next steps
    pub fn provide(&mut self) -> Result<Vec<S>, T::EarlyCutoff> {
        self.provider.provide(&self.timer, &self.state)
    }

    /// Decide which step to take (**must** be selected from among the ones that
    /// are provided by the [`provide`] function)
    pub fn decide(&mut self, step: S) {
        match &mut self.history {
            None => (),
            Some(his) => his.push(self.state.clone()),
        };
        self.state = step.apply(&self.state).unwrap();
    }

    /// Returns the current working expression
    pub fn working_expression(&self) -> &S::Exp {
        &self.state
    }

    /// Returns whether or not the current working expression is valid
    pub fn valid(&self) -> bool {
        self.checker.check(&self.state)
    }

    /// Returns whether or not meta-level "undo" is applicable
    pub fn can_undo(&self) -> bool {
        match &self.history {
            None => false,
            Some(xs) => !xs.is_empty(),
        }
    }

    /// Perform a meta-level "undo" operation
    ///
    /// # Panics
    ///
    /// Panics if "undo" is not applicable (can be checked with
    /// [`Self::can_undo`])
    pub fn undo(&mut self) {
        self.state = self.history.as_mut().unwrap().pop().unwrap();
    }
}
