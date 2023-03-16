# DAO Escrow

This is an adapted version of the Juno Unity contract to allow DAOs with non-staked treasuries to protect them against an attacker draining funds.

The withdrawal address:

1. Is the only address that can request a withdraw of funds
2. Is the only address that funds can be sent to
3. Has to wait for a cooldown to complete to execute the funds withdrawal

The assumption made by this contract is that the `withdrawal_address` would be a DAO that owns this contract, and escrows its treasury here, if that treasury is not staked. This means that even in the event of a VP attack, these funds cannot be moved by an attacker. NOTE that this does not protect against the minter being compromised on any CW20 contract associated with the owner DAO.

At any time, the withdraw can be cancelled by the `override_address`.

The `withdrawal_address` should be the DAO that seeks to escrow its treasury.

It is envisaged that the `override_address` should be:

- a developer multisig
- a SubDAO of the main DAO escrowing funds
- a smart contract with custom functionality

## Developing

Develop features and write unit tests.

Before committing, run `./scripts/check.sh`.

## Structure

The contract assumes a low-trust environment where the `withdrawal_address` might want to withdraw their funds.

However, there is a configurable delay to do so.

### Instantiation

The instantiation message has several configurable parameters.

```rs
pub struct InstantiateMsg {
    pub enable_cw20_receive: bool, // should the cw20 receive functionality be enabled? (cannot be changed later)
    pub set_withdraw_as_immutable: bool, // should the withdraw_address be updateable? (cannot be changed later)
    pub override_address: String,        // the deadman switch address and admin
    pub withdraw_address: String,        // the address whose funds are locked in this contract
    pub withdraw_delay_in_days: u64,     // withdraw delay in days
    pub native_denom: String,            // native chain denom - presumably ujuno
}
```

If in doubt, you should set `enable_cw20_receive` to `false` and `set_withdraw_as_immutable` to false. ONLY the `override_address` can change the `withdraw_address` or `override_address` at a later date.

### Withdraw

The `withdraw_address` has only one action available, withdrawing funds, on a timer:

This is implemented via two messages:

1. The first initiates a withdrawal. This call must specify the amount as well as the denom or contract address of the asset requested.
2. The second claims a withdrawal, if available. This also must match the request made in part 1. Why? Pedantry, as much as security.

The second action has three "flavours," depending on what's in the treasury of this contract. The three versions are:

1. Native balances (`ExecuteNativeWithdraw`)
2. CW20 balances (`ExecuteCW20Withdraw`)
3. Fully escrowed CW20 balances† (`ExecuteEscrowCW20Withdraw`)

† This contract implements the CW20 Receive interface, so it can store CW20s in its treasury if the `enable_cw20_receive` flag is set to `true`. Withdrawing CW20s has to be to a contract that also implements the Receive interface. This is considered an advanced feature, and honestly you probably shouldn't use it.

When a withdraw has been executed, the timer _will be reset_. Consider this when planning how to move balances.

### Override

The `override_address` has three actions available:

1. Cancel a pending withdrawal
2. Update the `override_address`
3. Update the `withdrawal_address`

Note that the override address does not have permission to withdraw funds. For this reason it should be a trusted address that will not set the withdraw address to itself and conduct an attack.

For this reason `withdrawal_address` can be set as immutable on instantiate, if required.

To do this, set `set_withdraw_as_immutable` to `true` on instantiation.
