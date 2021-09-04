# zkp-curveV1-circuit
Rewrite **Curve V1** using *Rust* and *Plonk Circuit*.
## Overview
[Curve](https://www.curve.fi/) is an exchange liquidity pool on Ethereum designed for extremely efficient stablecoin trading and low risk, supplemental fee income for liquidity providers, without an opportunity cost.

To implement layer2's Curve, I need to implement the Rust and Circuit versions of the Curve algorithm.
## Implementation
### Curve Source
 docs: [https://curve.readthedocs.io/_/downloads/en/latest/pdf/](https://curve.readthedocs.io/_/downloads/en/latest/pdf/)

 code: [https://github.com/curvefi/curve-contract/blob/master/contracts/pool-templates/base/SwapTemplateBase.vy#L674](https://github.com/curvefi/curve-contract/blob/master/contracts/pool-templates/base/SwapTemplateBase.vy#L674)
### Algorithm core invariant 
### Details
1. calculate D by [Newton-Raphson](https://en.wikipedia.org/wiki/Newton%27s_method) method
####
