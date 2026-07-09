# Emmy Noether

*Noise-aware quantum compiler passes based on fractal / long-memory noise modeling*

A noise-aware compiler building block for quantum circuits: instead of
optimizing routing/scheduling and dynamical-decoupling sequences purely on
gate count or a fixed pulse grid, a **time-varying (local) Hurst exponent**
is estimated from calibration data (Detrended Fluctuation Analysis), and an
adapted DD sequence as well as a noise-aware fidelity cost function are
derived from it.


## Architecture

```
emmy-noether/
├── core/                    Rust core library (no Python required)
│   └── src/
│       ├── hurst.rs         DFA estimation of the (local) Hurst exponent
│       ├── noise_gen.rs     synthetic colored noise (for testing)
│       ├── dd_synth.rs      Hurst-dependent DD sequence synthesis
│       ├── cost.rs          noise-aware fidelity/cost function
│       └── lib.rs           high-level API, ties all modules together
├── pybindings/               PyO3 bindings -> Python module `emmy_noether`
│   └── src/lib.rs
├── examples/
│   └── qiskit_pass.py       a real Qiskit TransformationPass that uses
│                             the Rust engine (demonstrated working)
└── README.md                 this document
```

## Building and Testing

Core library (pure Rust, 12 unit/integration tests):

```bash
cd core
cargo test --release
```

Python bindings (PyO3, requires Python dev headers):

```bash
cd pybindings
cargo build --release
# Since this is a Cargo workspace, the output lands in the workspace root:
# produces ../target/release/libemmy_noether.so
# rename/copy it as emmy_noether.so for the Python import
# (in production, use `pip install maturin && maturin build --release`,
# which handles naming/wheel packaging automatically)
```

Qiskit integration example (requires Qiskit >= 2.0):

```bash
cd examples
cp ../target/release/libemmy_noether.so ./emmy_noether.so
python3 qiskit_pass.py
```

All three steps were actually executed while building this prototype and
work end-to-end (12/12 Rust tests passing, PyO3 module importable, Qiskit
pass runs and visibly inserts pulses into the circuit).

## What Is ACTUALLY Validated Here -- and What Is Not

**Validated (via tests/execution in this PoC):**
- The DFA implementation reliably distinguishes between synthetic white
  noise (H ~ 0.5) and synthetic persistent noise (target H 0.9 -> estimated
  H clearly higher).
- The DD synthesis produces larger pulse spacing for higher estimated H and
  respects the hardware minimum pulse gap.
- The entire pipeline (calibration series -> H estimation -> DD sequence ->
  Qiskit pass) runs through technically end-to-end.

**NOT validated (this is the honest core to keep in mind before turning this
into a product or a grant application):**
- The relationship H -> pulse spacing (`dd_synth.rs`) is a plausible but
  self-constructed heuristic -- not an optimal sequence derived from a
  filter-function analysis.
- The stretching exponent p(H) = 2H in `cost.rs` is a simplifying model
  assumption, not a formula taken from the cited papers.
- Only SYNTHETIC test data has been used so far, no real calibration
  measurements from actual hardware (AQT, IBM, etc.).
- There is not yet a comparison against a real baseline compiler pass (e.g.
  Qiskit's PadDynamicalDecoupling) on real or simulated hardware with a
  noise model.

## Sensible Next Steps

1. Test the DFA estimation against real Ramsey/echo calibration data (e.g.
   from AQT or an IBM backend via Qiskit Runtime) -- does the estimated
   H(t) match published values for the respective platform?
2. Replace the DD synthesis heuristic with an actual filter-function
   optimization, or at least benchmark it against one.
3. Seek collaboration with a noise-characterization group (e.g. IQOQI
   Innsbruck) to have the model assumptions reviewed by domain experts
   before turning this into a grant application or company narrative.
4. Real benchmark against Qiskit's `PadDynamicalDecoupling` and/or pytket's
   DD passes on a simulator with a realistic 1/f noise model.

In short: this is a working technical skeleton that shows HOW the idea can
be built into a compiler pipeline -- the physical proof that the idea
actually delivers better fidelity than standard DD is still outstanding,
and is the next, most important step.
