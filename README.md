# Solana Security Workshop

Welcome to our Solana Security Workshop!

All details are in the docs. To check it out online, visit [https://workshop.neodyme.io](https://workshop.neodyme.io).

To build it yourself, install mdbook (`cargo install mdbook`) and run `mdbook serve`.


View Docs Online: [Workshop Docs](https://workshop.neodyme.io/index.html)


### Local Development

Compile All Contracts: `cargo build-bpf --workspace`

Run an exploit: `RUST_BACKTRACE=1 cargo run --bin level{insert_level_#_here}`

### Completed Challenges

#### Level 0

Exploited by creating manually creating a Wallet with the victim's vault, but the hacker's public key.

Execute solution: `cargo build-bpf --workspace && RUST_BACKTRACE=1 cargo run --bin level0`

#### Level 1

Initial thoughts:
- directly setting the authority won't achieve anything.
- possible to re-use a transaction?

Execute solution: `cargo build-bpf --workspace && RUST_BACKTRACE=1 cargo run --bin level1`

#### Level 2

Initial thoughts:
- re-initialize the account to hijack the authority (doesn't work afaik)

Execute solution: `cargo build-bpf --workspace && RUST_BACKTRACE=1 cargo run --bin level2`

#### Level 3

Initial thoughts:
- Send an overflow by depositing a tip into another pool
  - Doesn't work since there are checks on pool and vault owner's being the program id
- 

Execute solution: `cargo build-bpf --workspace && RUST_BACKTRACE=1 cargo run --bin level3`


#### Level 4

Initial thoughts:


Execute solution: `cargo build-bpf --workspace && RUST_BACKTRACE=1 cargo run --bin level4`