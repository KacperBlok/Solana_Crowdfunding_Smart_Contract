# Solana Crowdfunding Smart Contract

[![Solana](https://img.shields.io/badge/Solana-9945FF?style=flat&logo=solana&logoColor=white)](https://solana.com/)
[![Anchor](https://img.shields.io/badge/Anchor-512DA8?style=flat&logo=anchor&logoColor=white)](https://www.anchor-lang.com/)
[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

> A decentralized crowdfunding platform built on Solana blockchain using the Anchor framework

## 📋 Table of Contents
- [🌟 Overview](#-overview)
- [🏗️ Architecture](#️-architecture)
- [⚡ Features](#-features)
- [📊 Data Structures](#-data-structures)
- [🔧 Instructions](#-instructions)
- [🔒 Security](#-security)
- [🔑 PDA Mechanism](#-pda-mechanism)
- [📡 Events](#-events)
- [❌ Error Handling](#-error-handling)
- [💻 Usage](#-usage)
- [📈 Conclusions](#-conclusions)

## 🌟 Overview

This smart contract implements a crowdfunding platform on the Solana blockchain using the Anchor framework. It enables creating crowdfunding campaigns, accepting contributions from participants, fund withdrawals for campaign creators, and refund processing in case of campaign failure.

### ✨ Key Features:
- **Decentralization**: Completely decentralized platform without central authority
- **Transparency**: All transactions and campaign states are public on the blockchain
- **Automation**: Business logic is enforced by smart contract
- **Security**: Utilizes Solana and Anchor security mechanisms
- **Scalability**: Supports unlimited number of campaigns and participants

## 🏗️ Architecture

### 🔧 System Components:

1. **Main Program** (`crowdfunding`): Contains all business logic
2. **PDA Accounts** (Program Derived Addresses): Deterministic addresses for campaigns and contributions
3. **Token Accounts**: SPL Token fund storage
4. **Events**: Event emission for frontend and monitoring

### 📋 Data Model:

```
Campaign
├── Metadata (title, description, goal)
├── Time parameters (start, end)
├── Financial state (raised/target)
├── Status flags (success, withdrawn)
└── Statistics (participant count)

Contribution
├── Participant identifier
├── Campaign association
└── Contribution amount
```

## ⚡ Features

### 1️⃣ Campaign Creation (`initialize_campaign`)

**Purpose**: Enables users to create new crowdfunding campaigns.

**Parameters**:
- `title`: Campaign title (max 100 characters)
- `description`: Campaign description (max 500 characters)  
- `target_amount`: Target amount to raise
- `duration_days`: Campaign duration (1-365 days)

**Process**:
1. Input parameter validation
2. Create Campaign account with unique PDA
3. Create vault (token account) for fund storage
4. Initialize all campaign fields
5. Emit `CampaignCreated` event

**PDA Mechanism**:
- **Campaign PDA**: `[b"campaign", creator.key(), title.as_bytes()]`
- **Vault PDA**: `[b"vault", campaign.key()]`

### 2️⃣ Fund Contribution (`contribute`)

**Purpose**: Enables users to contribute funds to selected campaigns.

**Parameters**:
- `amount`: Amount to contribute

**Validations**:
- Campaign must be active (not ended)
- Amount greater than zero
- Funds not yet withdrawn
- Contribution doesn't exceed campaign goal

**Process**:
1. Check contribution conditions
2. Transfer tokens from participant account to campaign vault
3. Update/create Contribution record
4. Update campaign state (raised amount, participant count)
5. Check if goal reached (set success flag)
6. Emit `ContributionMade` event

**Participant Counting Mechanism**:
- If `contribution.amount == 0` → new participant
- Increase `contributors_count` only for new participants

### 3️⃣ Fund Withdrawal (`withdraw_funds`)

**Purpose**: Enables campaign creator to withdraw raised funds.

**Withdrawal Conditions**:
- Only campaign creator can withdraw funds
- Campaign must be successful OR time expired
- Funds not yet withdrawn
- Vault must contain funds to withdraw

**Process**:
1. Verify permissions and conditions
2. Calculate withdrawal amount
3. Transfer all funds from vault to creator account
4. Set `is_withdrawn = true` flag
5. Emit `FundsWithdrawn` event

**Note**: Withdrawal transfers **all** funds from vault, not just `current_amount`.

### 4️⃣ Contribution Refund (`refund_contribution`)

**Purpose**: Enables participants to recover funds from failed campaigns.

**Refund Conditions**:
- Campaign must be time-expired
- Campaign cannot be marked as successful
- Participant must have non-zero contribution

**Process**:
1. Verify refund conditions
2. Get refund amount from Contribution record
3. Transfer funds from vault to participant account
4. Zero out participant's contribution
5. Emit `ContributionRefunded` event

## 📊 Data Structures

### 🏢 Campaign
```rust
pub struct Campaign {
    pub creator: Pubkey,           // Campaign creator (32 bytes)
    pub title: String,             // Title (4 + 100 bytes)
    pub description: String,       // Description (4 + 500 bytes)
    pub target_amount: u64,        // Financial goal (8 bytes)
    pub current_amount: u64,       // Raised amount (8 bytes)
    pub start_time: i64,           // Start time (8 bytes)
    pub end_time: i64,             // End time (8 bytes)
    pub is_successful: bool,       // Goal achieved (1 byte)
    pub is_withdrawn: bool,        // Funds withdrawn (1 byte)
    pub contributors_count: u32,   // Participant count (4 bytes)
}
```
**Total size**: 578 bytes

### 💰 Contribution
```rust
pub struct Contribution {
    pub contributor: Pubkey,       // Participant (32 bytes)
    pub campaign: Pubkey,          // Campaign (32 bytes)
    pub amount: u64,               // Contribution amount (8 bytes)
}
```
**Total size**: 80 bytes

## 🔧 Instructions

### 📝 Instruction Contexts

#### InitializeCampaign
- **campaign**: New campaign account (PDA)
- **campaign_vault**: New token account (PDA) 
- **creator**: Signer and payer
- **mint**: SPL token account
- **Programs**: Token, System, Rent

#### Contribute  
- **campaign**: Existing campaign account
- **contribution**: Contribution account (init_if_needed)
- **campaign_vault**: Campaign vault
- **contributor_token_account**: Participant token account
- **contributor**: Participant signer
- **Programs**: Token, System, Rent

#### WithdrawFunds
- **campaign**: Campaign account
- **campaign_vault**: Campaign vault  
- **creator_token_account**: Creator token account
- **creator**: Creator signer
- **Programs**: Token

#### RefundContribution
- **campaign**: Campaign account
- **contribution**: Participant contribution account
- **campaign_vault**: Campaign vault
- **contributor_token_account**: Participant token account  
- **contributor**: Participant signer
- **Programs**: Token

## 🔒 Security

### 🛡️ Protection Mechanisms:

1. **Access Control**:
   - Only creator can withdraw funds
   - Only contribution owner can request refund

2. **State Validation**:
   - Campaign activity verification
   - Withdrawal/refund condition checks
   - Arithmetic overflow protection

3. **Double-spending Protection**:
   - `is_withdrawn` flag prevents multiple withdrawals
   - Zero `contribution.amount` after refund

4. **Parameter Validation**:
   - String length limitations
   - Amount and time validity checks
   - Maximum campaign duration control

5. **Operation Atomicity**:
   - All operations are atomic
   - Partial failure causes complete rollback

### ⚠️ Potential Threats and Mitigations:

- **Overflow attacks**: Use of `checked_add()` and `checked_mul()`
- **Reentrancy**: No external calls after state changes
- **Authorization bypass**: Explicit checks on `creator.key()`
- **State manipulation**: Immutable references where possible

## 🔑 PDA Mechanism

Program Derived Addresses (PDA) provide deterministic and secure addresses:

### 🌱 Seeds Used:

1. **Campaign PDA**: 
   ```
   seeds = [b"campaign", creator.key(), title.as_bytes()]
   ```
   - Uniqueness: creator + title
   - Prevents duplicate campaigns from same creator

2. **Vault PDA**:
   ```
   seeds = [b"vault", campaign.key()]
   ```
   - One vault per campaign
   - Automatic fund management

3. **Contribution PDA**:
   ```
   seeds = [b"contribution", campaign.key(), contributor.key()]
   ```
   - One contribution record per participant per campaign
   - Enables multiple contributions from same participant

### ✅ PDA Advantages:
- **Determinism**: Addresses are predictable
- **Security**: Program has exclusive control
- **Efficiency**: No need to store additional keys
- **Scalability**: Unlimited number of accounts

## 📡 Events

Event emission system enables activity monitoring:

### 🎉 CampaignCreated
```rust
pub struct CampaignCreated {
    pub campaign: Pubkey,      // Campaign address
    pub creator: Pubkey,       // Creator
    pub target_amount: u64,    // Financial goal
    pub end_time: i64,         // End time
}
```

### 💳 ContributionMade  
```rust
pub struct ContributionMade {
    pub campaign: Pubkey,      // Campaign
    pub contributor: Pubkey,   // Participant
    pub amount: u64,           // Contribution amount
    pub total_raised: u64,     // Total raised amount
}
```

### 💸 FundsWithdrawn
```rust
pub struct FundsWithdrawn {
    pub campaign: Pubkey,      // Campaign
    pub creator: Pubkey,       // Creator
    pub amount: u64,           // Withdrawn amount
}
```

### 🔄 ContributionRefunded
```rust
pub struct ContributionRefunded {
    pub campaign: Pubkey,      // Campaign  
    pub contributor: Pubkey,   // Participant
    pub amount: u64,           // Refunded amount
}
```

## ❌ Error Handling

Complete error handling system with descriptive messages:

### ⚠️ Validation Errors:
- `TitleTooLong`: Title exceeds 100 characters
- `DescriptionTooLong`: Description exceeds 500 characters  
- `InvalidTargetAmount`: Invalid target amount
- `InvalidDuration`: Duration outside 1-365 days range

### 📅 Campaign State Errors:
- `CampaignEnded`: Contribution attempt after end
- `CampaignStillActive`: Refund attempt on active campaign
- `CampaignWasSuccessful`: Refund attempt on successful campaign
- `CampaignAlreadyWithdrawn`: Contribution attempt after withdrawal

### 💰 Financial Errors:
- `InvalidContributionAmount`: Invalid contribution amount
- `ExceedsTarget`: Contribution exceeds campaign goal
- `AmountOverflow`: Arithmetic overflow
- `NoFundsToWithdraw`: No funds available for withdrawal
- `NoContributionToRefund`: No contribution to refund

### 🔐 Authorization Errors:
- `UnauthorizedWithdrawal`: Unauthorized withdrawal attempt
- `WithdrawalConditionsNotMet`: Withdrawal conditions not met
- `AlreadyWithdrawn`: Funds already withdrawn

## 💻 Usage

### 🚀 Example Flow:

1. **Campaign Creation**:
   ```typescript
   await program.methods
     .initializeCampaign("My Campaign", "Campaign description", new BN(1000000), 30)
     .accounts({ /* accounts */ })
     .rpc();
   ```

2. **Fund Contribution**:
   ```typescript
   await program.methods
     .contribute(new BN(100000))
     .accounts({ /* accounts */ })
     .rpc();
   ```

3. **Fund Withdrawal** (after success):
   ```typescript
   await program.methods
     .withdrawFunds()
     .accounts({ /* accounts */ })
     .rpc();
   ```

4. **Contribution Refund** (after failure):
   ```typescript
   await program.methods
     .refundContribution()
     .accounts({ /* accounts */ })
     .rpc();
   ```

### 🔄 Transaction Flow:
```
Client Request → Parameter Validation → Account Verification → 
Business Logic → State Update → Event Emission → Response
```

## 📈 Conclusions

This smart contract presents a comprehensive crowdfunding solution for the Solana ecosystem, implementing security and architecture best practices. It utilizes advanced features of the Anchor framework and Solana mechanisms to provide a secure, scalable, and efficient crowdfunding platform.

### 🎯 Key Achievements:
- **Security**: Comprehensive protection mechanisms
- **Transparency**: Full auditability of all operations  
- **Efficiency**: Optimal utilization of Solana resources
- **Scalability**: Architecture enabling unlimited growth
- **Usability**: Intuitive API for frontend developers

### 🚀 Development Possibilities:
- Implementation of partial refunds
- Platform fee system
- Staking mechanisms for verification
- Oracle integration for currency rates
- Campaign creator reputation system
- Multi-token support
- Advanced governance features

### 💡 Production Considerations:
- **Monitoring**: Implement comprehensive event tracking
- **Upgrades**: Plan for program upgrade strategies
- **Performance**: Optimize for high-throughput scenarios
- **Compliance**: Consider regulatory requirements
- **Integration**: Design for ecosystem compatibility

This implementation serves as a robust foundation for decentralized crowdfunding applications on Solana, providing the security, scalability, and functionality required for real-world blockchain applications.
