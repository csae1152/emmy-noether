# Emmy Noether

*Noise-aware quantum compiler passes based on fractal / long-memory noise modeling*

Ein noise-aware Compiler-Baustein fuer Quantenschaltkreise: statt Routing/
Scheduling und Dynamical-Decoupling-Sequenzen nur auf Gate-Anzahl bzw. ein
starres Pulsraster zu optimieren, wird ein **zeitvariabler (lokaler)
Hurst-Exponent** aus Kalibrierungsdaten geschaetzt (Detrended Fluctuation
Analysis) und daraus eine angepasste DD-Sequenz sowie eine rauschbewusste
Fidelity-Kostenfunktion abgeleitet.

Hintergrund und Motivation: siehe Chatverlauf zu "memory multi-fractional
Brownian motion" (mmfBm) in supraleitenden/Ionenfallen-Qubits und die
Ionenfallen-/AQT-/Quantum-Hub-Tirol-Diskussion (Innsbruck).

## Architektur

```
emmy-noether/
├── core/                    Rust-Kernbibliothek (kein Python noetig)
│   └── src/
│       ├── hurst.rs         DFA-Schaetzung des (lokalen) Hurst-Exponenten
│       ├── noise_gen.rs     synthetisches farbiges Rauschen (fuer Tests)
│       ├── dd_synth.rs      Hurst-abhaengige DD-Sequenz-Synthese
│       ├── cost.rs          rauschbewusste Fidelity-/Kostenfunktion
│       └── lib.rs           High-Level-API, verbindet alle Module
├── pybindings/               PyO3-Bindings -> Python-Modul `emmy_noether`
│   └── src/lib.rs
├── examples/
│   └── qiskit_pass.py       echter Qiskit TransformationPass, der die
│                             Rust-Engine nutzt (lauffaehig demonstriert)
└── README.md                 dieses Dokument
```

## Bauen und Testen

Core-Bibliothek (reines Rust, 12 Unit-/Integrationstests):

```bash
cd core
cargo test --release
```

Python-Bindings (PyO3, benoetigt Python-Dev-Header):

```bash
cd pybindings
cargo build --release
# Da dies ein Cargo-Workspace ist, landet der Output im Workspace-Root:
# erzeugt ../target/release/libemmy_noether.so
# fuer den Python-Import umbenennen/kopieren als emmy_noether.so
# (produktiv: `pip install maturin && maturin build --release` verwenden,
# das erledigt Benennung/Wheel-Verpackung automatisch)
```

Qiskit-Integrationsbeispiel (setzt Qiskit >= 2.0 voraus):

```bash
cd examples
cp ../target/release/libemmy_noether.so ./emmy_noether.so
python3 qiskit_pass.py
```

Alle drei Schritte wurden beim Erstellen dieses Prototyps tatsaechlich
ausgefuehrt und funktionieren End-to-End (12/12 Rust-Tests gruen, PyO3-Modul
importierbar, Qiskit-Pass laeuft und fuegt sichtbar Pulse in den Schaltkreis ein).

## Was hier WIRKLICH validiert ist -- und was nicht

**Validiert (durch Tests/Ausfuehrung in diesem PoC):**
- Die DFA-Implementierung unterscheidet zuverlaessig zwischen synthetischem
  weissem Rauschen (H ~ 0.5) und synthetischem persistentem Rauschen (Ziel-H
  0.9 -> geschaetztes H deutlich hoeher).
- Die DD-Synthese erzeugt bei hoeherem geschaetzten H tatsaechlich groessere
  Pulsabstaende und respektiert die Hardware-Mindestpulsluecke.
- Die gesamte Pipeline (Kalibrierungsreihe -> H-Schaetzung -> DD-Sequenz ->
  Qiskit-Pass) laeuft technisch durchgaengig durch.

**NICHT validiert (das ist der ehrliche Kern, bevor man daraus ein Produkt
oder einen Foerderantrag macht):**
- Der Zusammenhang H -> Pulsabstand (`dd_synth.rs`) ist eine plausible,
  aber selbst konstruierte Heuristik -- keine aus einer Filterfunktionsanalyse
  hergeleitete optimale Sequenz.
- Der Streckungsexponent p(H) = 2H in `cost.rs` ist eine vereinfachende
  Modellannahme, keine aus den zitierten Papers uebernommene Formel.
- Es wurden ausschliesslich SYNTHETISCHE Testdaten verwendet, keine echten
  Kalibrierungsmessungen von realer Hardware (AQT, IBM, o.ae.).
- Es gibt noch keinen Vergleich gegen einen echten Baseline-Compiler-Pass
  (z.B. Qiskit's PadDynamicalDecoupling) auf echter oder simulierter
  Hardware mit Rauschmodell.

## Naechste sinnvolle Schritte

1. Mit echten Ramsey-/Echo-Kalibrierungsdaten (z.B. von AQT oder einem IBM-
   Backend ueber Qiskit Runtime) die DFA-Schaetzung testen -- passt das
   geschaetzte H(t) zu publizierten Werten fuer die jeweilige Plattform?
2. Die DD-Synthese-Heuristik durch eine echte Filterfunktions-Optimierung
   ersetzen oder zumindest dagegen benchmarken.
3. Kooperation mit einer Rauschcharakterisierungs-Gruppe (z.B. IQOQI
   Innsbruck) suchen, um die Modellannahmen fachlich zu pruefen, bevor
   daraus eine Foerderantrag-/Firmen-Erzaehlung wird.
4. Reales Benchmark gegen `PadDynamicalDecoupling` (Qiskit) und/oder pytket's
   DD-Passes auf einem Simulator mit realistischem 1/f-Rauschmodell.

Kurz: Das hier ist ein funktionierendes technisches Skelett, das zeigt, WIE
sich die Idee in eine Compiler-Pipeline einbauen laesst -- der physikalische
Beweis, dass die Idee tatsaechlich bessere Fidelity liefert als Standard-DD,
steht noch aus und ist der naechste, wichtigste Schritt.
