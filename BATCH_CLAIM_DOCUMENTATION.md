# Batch Claim Functionality

## Overview

The `batch_claim` function allows advisors to claim tokens from multiple vesting schedules (e.g., Seed, Private, Advisory) in a single transaction, significantly reducing gas costs and improving user experience.

## Problem Solved

Previously, advisors with multiple vesting schedules had to:
1. Call `claim_tokens` separately for each vault
2. Pay gas fees for each transaction
3. Track multiple transactions manually

## Solution

The `batch_claim` function:
- Aggregates available tokens across all schedules linked to a single address
- Executes a single token transfer for the total amount
- Maintains all existing security checks and validations
- Updates all vault states atomically

## Functions Added

### `batch_claim(env: Env) -> i128`

**Description**: Claims all available tokens from all vaults owned by the caller.

**Parameters**:
- `env: Env` - The contract environment

**Returns**:
- `i128` - Total amount of tokens claimed

**Behavior**:
1. Gets all vault IDs for the caller using `get_user_vaults`
2. Iterates through each vault and calculates claimable amount
3. Skips frozen, uninitialized, or paused vaults
4. Respects locked tokens (collateral liens)
5. Updates all vault states atomically
6. Performs single token transfer for total amount
7. Mints NFT once if configured

### `get_total_claimable_amount(env: Env, user: Address) -> i128`

**Description**: Returns the total claimable amount across all user's vaults without claiming.

**Parameters**:
- `env: Env` - The contract environment
- `user: Address` - The user address to check

**Returns**:
- `i128` - Total claimable amount across all vaults

## Gas Optimization Benefits

1. **Single Transaction**: Instead of N transactions for N vaults, only 1 transaction needed
2. **Single Token Transfer**: One transfer operation instead of multiple
3. **Single NFT Mint**: NFT minted only once per batch claim (if configured)
4. **Reduced Storage Operations**: Batched vault state updates

## Security Considerations

1. **Authentication**: Uses `env.invoker()` and requires authentication
2. **Pause Checks**: Respects global pause and individual vault pause states
3. **Vault Validation**: Skips frozen, uninitialized, or invalid vaults
4. **Locked Tokens**: Respects collateral liens and locked amounts
5. **Atomic Updates**: All vault states updated atomically

## Usage Examples

### Basic Usage

```rust
// User calls batch_claim to claim from all their vaults
let claimed_amount = contract.batch_claim();
println!("Claimed {} tokens", claimed_amount);
```

### Checking Available Amounts

```rust
// Check total claimable before claiming
let available = contract.get_total_claimable_amount(user_address);
if available > 0 {
    let claimed = contract.batch_claim();
    assert_eq!(claimed, available);
}
```

## Testing

Comprehensive test suite added in `tests/batch_claim.rs`:

1. **Single Vault Claim**: Basic functionality test
2. **Multiple Vaults Claim**: Aggregation test (Seed, Private, Advisory scenario)
3. **Frozen Vault Handling**: Skips frozen vaults correctly
4. **Paused Vault Handling**: Respects pause states
5. **No Claimable Tokens**: Handles edge cases
6. **No Vaults**: Handles users with no vaults
7. **Locked Tokens**: Respects collateral liens

## Backward Compatibility

- All existing functions remain unchanged
- No breaking changes to existing API
- Existing `claim_tokens` function still works for single vault claims

## Gas Savings Estimation

For an advisor with 3 vesting schedules:
- **Before**: 3 transactions × (claim + transfer + NFT mint operations)
- **After**: 1 transaction × (batch claim + single transfer + single NFT mint)

Estimated gas savings: ~60-70% reduction in transaction costs.

## Future Enhancements

Potential future improvements:
1. **Selective Batch Claim**: Allow claiming from specific vaults only
2. **Claim Scheduling**: Allow scheduling batch claims for future timestamps
3. **Claim History**: Track batch claim events for better analytics
