# Solana Crowdfunding Smart Contract

[![Solana](https://img.shields.io/badge/Solana-9945FF?style=flat&logo=solana&logoColor=white)](https://solana.com/)
[![Anchor](https://img.shields.io/badge/Anchor-512DA8?style=flat&logo=anchor&logoColor=white)](https://www.anchor-lang.com/)
[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

> Zdecentralizowana platforma crowdfundingowa zbudowana na blockchainie Solana przy użyciu frameworka Anchor

## 📋 Spis treści
- [🌟 Przegląd](#-przegląd)
- [🏗️ Architektura](#️-architektura)
- [⚡ Funkcjonalności](#-funkcjonalności)
- [📊 Struktury danych](#-struktury-danych)
- [🔧 Instrukcje](#-instrukcje)
- [🔒 Bezpieczeństwo](#-bezpieczeństwo)
- [🔑 Mechanizm PDA](#-mechanizm-pda)
- [📡 Wydarzenia](#-wydarzenia)
- [❌ Obsługa błędów](#-obsługa-błędów)
- [💻 Użycie](#-użycie)
- [🧪 Testowanie](#-testowanie)
- [📈 Wnioski](#-wnioski)

## 🌟 Przegląd

Ten smart contract implementuje platformę crowdfundingową na blockchainie Solana przy użyciu frameworka Anchor. Umożliwia tworzenie kampanii crowdfundingowych, przyjmowanie wpłat od uczestników, wypłacanie środków twórcom kampanii oraz zwracanie środków w przypadku niepowodzenia kampanii.

### ✨ Kluczowe cechy:
- **Decentralizacja**: Kompletnie zdecentralizowana platforma bez centralnej władzy
- **Przejrzystość**: Wszystkie transakcje i stany kampanii są publiczne na blockchainie
- **Automatyzacja**: Logika biznesowa jest egzekwowana przez smart contract
- **Bezpieczeństwo**: Wykorzystuje mechanizmy bezpieczeństwa Solany i Anchor
- **Skalowalność**: Obsługuje nieograniczoną liczbę kampanii i uczestników

## 🏗️ Architektura

### 🔧 Komponenty systemu:

1. **Program główny** (`crowdfunding`): Zawiera całą logikę biznesową
2. **Konta PDA** (Program Derived Addresses): Deterministyczne adresy dla kampanii i wkładów
3. **Token Accounts**: Przechowywanie środków SPL Token
4. **Wydarzenia**: Emisja zdarzeń dla frontend i monitoring

### 📋 Model danych:

```
Campaign (Kampania)
├── Metadane (tytuł, opis, cel)
├── Parametry czasowe (start, koniec)
├── Stan finansowy (zebrane/cel)
├── Flagi statusu (sukces, wypłacono)
└── Statystyki (liczba uczestników)

Contribution (Wkład)
├── Identyfikator uczestnika
├── Powiązanie z kampanią
└── Kwota wkładu
```

## ⚡ Funkcjonalności

### 1️⃣ Tworzenie kampanii (`initialize_campaign`)

**Cel**: Umożliwia użytkownikom tworzenie nowych kampanii crowdfundingowych.

**Parametry**:
- `title`: Tytuł kampanii (max 100 znaków)
- `description`: Opis kampanii (max 500 znaków)  
- `target_amount`: Docelowa kwota do zebrania
- `duration_days`: Czas trwania kampanii (1-365 dni)

**Proces**:
1. Walidacja parametrów wejściowych
2. Utworzenie konta Campaign z unikalnym PDA
3. Utworzenie vault'a (konta tokenowego) dla przechowywania środków
4. Inicjalizacja wszystkich pól kampanii
5. Emisja wydarzenia `CampaignCreated`

**Mechanizm PDA**:
- **Campaign PDA**: `[b"campaign", creator.key(), title.as_bytes()]`
- **Vault PDA**: `[b"vault", campaign.key()]`

### 2️⃣ Wpłacanie środków (`contribute`)

**Cel**: Umożliwia użytkownikom wpłacanie środków na wybraną kampanię.

**Parametry**:
- `amount`: Kwota do wpłacenia

**Walidacje**:
- Kampania musi być aktywna (nie zakończona czasowo)
- Kwota większa od zera
- Środki nie zostały jeszcze wypłacone
- Wpłata nie przekracza celu kampanii

**Proces**:
1. Sprawdzenie warunków wpłaty
2. Transfer tokenów z konta uczestnika do vault'a kampanii
3. Aktualizacja/utworzenie rekordu Contribution
4. Aktualizacja stanu kampanii (zebrana kwota, liczba uczestników)
5. Sprawdzenie czy osiągnięto cel (ustawienie flagi sukcesu)
6. Emisja wydarzenia `ContributionMade`

**Mechanizm liczenia uczestników**:
- Jeśli `contribution.amount == 0` → nowy uczestnik
- Zwiększenie `contributors_count` tylko dla nowych uczestników

### 3️⃣ Wypłata środków (`withdraw_funds`)

**Cel**: Umożliwia twórcy kampanii wypłatę zebranych środków.

**Warunki wypłaty**:
- Tylko twórca kampanii może wypłacić środki
- Kampania musi być zakończona sukcesem LUB minął czas trwania
- Środki nie zostały jeszcze wypłacone
- W vault'ie muszą być środki do wypłaty

**Proces**:
1. Weryfikacja uprawnień i warunków
2. Obliczenie kwoty do wypłaty
3. Transfer całości środków z vault'a na konto twórcy
4. Ustawienie flagi `is_withdrawn = true`
5. Emisja wydarzenia `FundsWithdrawn`

**Uwaga**: Wypłata transferuje **wszystkie** środki z vault'a, nie tylko `current_amount`.

### 4️⃣ Zwrot środków (`refund_contribution`)

**Cel**: Umożliwia uczestnikom odzyskanie środków z nieudanych kampanii.

**Warunki zwrotu**:
- Kampania musi być zakończona czasowo
- Kampania nie może być oznaczona jako sukces
- Uczestnik musi mieć niezerowy wkład

**Proces**:
1. Weryfikacja warunków zwrotu
2. Pobranie kwoty do zwrotu z rekordu Contribution
3. Transfer środków z vault'a na konto uczestnika
4. Wyzerowanie wkładu uczestnika
5. Emisja wydarzenia `ContributionRefunded`

## 📊 Struktury danych

### 🏢 Campaign (Kampania)
```rust
pub struct Campaign {
    pub creator: Pubkey,           // Twórca kampanii (32 bytes)
    pub title: String,             // Tytuł (4 + 100 bytes)
    pub description: String,       // Opis (4 + 500 bytes)
    pub target_amount: u64,        // Cel finansowy (8 bytes)
    pub current_amount: u64,       // Zebrana kwota (8 bytes)
    pub start_time: i64,           // Czas rozpoczęcia (8 bytes)
    pub end_time: i64,             // Czas zakończenia (8 bytes)
    pub is_successful: bool,       // Czy osiągnięto cel (1 byte)
    pub is_withdrawn: bool,        // Czy wypłacono środki (1 byte)
    pub contributors_count: u32,   // Liczba uczestników (4 bytes)
}
```
**Całkowity rozmiar**: 578 bytes

### 💰 Contribution (Wkład)
```rust
pub struct Contribution {
    pub contributor: Pubkey,       // Uczestnik (32 bytes)
    pub campaign: Pubkey,          // Kampania (32 bytes)
    pub amount: u64,               // Kwota wkładu (8 bytes)
}
```
**Całkowity rozmiar**: 80 bytes

## 🔧 Instrukcje

### 📝 Konteksty instrukcji

#### InitializeCampaign
- **campaign**: Nowe konto kampanii (PDA)
- **campaign_vault**: Nowe konto tokenowe (PDA) 
- **creator**: Sygnatariusz i płatnik
- **mint**: Konto tokena SPL
- **Programy**: Token, System, Rent

#### Contribute  
- **campaign**: Istniejące konto kampanii
- **contribution**: Konto wkładu (init_if_needed)
- **campaign_vault**: Vault kampanii
- **contributor_token_account**: Konto tokenowe uczestnika
- **contributor**: Sygnatariusz uczestnika
- **Programy**: Token, System, Rent

#### WithdrawFunds
- **campaign**: Konto kampanii
- **campaign_vault**: Vault kampanii  
- **creator_token_account**: Konto tokenowe twórcy
- **creator**: Sygnatariusz twórcy
- **Programy**: Token

#### RefundContribution
- **campaign**: Konto kampanii
- **contribution**: Konto wkładu uczestnika
- **campaign_vault**: Vault kampanii
- **contributor_token_account**: Konto tokenowe uczestnika  
- **contributor**: Sygnatariusz uczestnika
- **Programy**: Token

## 🔒 Bezpieczeństwo

### 🛡️ Mechanizmy ochrony:

1. **Kontrola dostępu**:
   - Tylko twórca może wypłacić środki
   - Tylko właściciel wkładu może żądać zwrotu

2. **Walidacja stanu**:
   - Sprawdzanie aktywności kampanii
   - Weryfikacja warunków wypłaty/zwrotu
   - Kontrola przepełnienia arytmetycznego

3. **Ochrona przed double-spending**:
   - Flaga `is_withdrawn` zapobiega wielokrotnym wypłatom
   - Zerowanie `contribution.amount` po zwrocie

4. **Walidacja parametrów**:
   - Ograniczenia długości stringów
   - Sprawdzanie poprawności kwot i czasów
   - Kontrola maksymalnego czasu trwania kampanii

5. **Atomowość operacji**:
   - Wszystkie operacje są atomowe
   - Niepowodzenie jednej części powoduje rollback całej transakcji

### ⚠️ Potencjalne zagrożenia i mitygacje:

- **Overflow attacks**: Użycie `checked_add()` i `checked_mul()`
- **Reentrancy**: Brak wywołań zewnętrznych po zmianach stanu
- **Authorization bypass**: Explicit checks na `creator.key()`
- **State manipulation**: Immutable references gdzie to możliwe

## 🔑 Mechanizm PDA

Program Derived Addresses (PDA) zapewniają deterministyczne i bezpieczne adresy:

### 🌱 Seeds wykorzystywane:

1. **Campaign PDA**: 
   ```
   seeds = [b"campaign", creator.key(), title.as_bytes()]
   ```
   - Unikalność: creator + title
   - Zapobiega duplikatom kampanii od tego samego twórcy

2. **Vault PDA**:
   ```
   seeds = [b"vault", campaign.key()]
   ```
   - Jeden vault per kampania
   - Automatyczne zarządzanie środkami

3. **Contribution PDA**:
   ```
   seeds = [b"contribution", campaign.key(), contributor.key()]
   ```
   - Jeden rekord wkładu per uczestnik per kampania
   - Umożliwia wielokrotne wpłaty od tego samego uczestnika

### ✅ Zalety PDA:
- **Determinizm**: Adresy są przewidywalne
- **Bezpieczeństwo**: Program ma wyłączną kontrolę
- **Oszczędność**: Brak potrzeby przechowywania dodatkowych kluczy
- **Skalowalność**: Nieograniczona liczba kont

## 📡 Wydarzenia

System emisji zdarzeń umożliwia monitorowanie aktywności:

### 🎉 CampaignCreated
```rust
pub struct CampaignCreated {
    pub campaign: Pubkey,      // Adres kampanii
    pub creator: Pubkey,       // Twórca
    pub target_amount: u64,    // Cel finansowy
    pub end_time: i64,         // Czas zakończenia
}
```

### 💳 ContributionMade  
```rust
pub struct ContributionMade {
    pub campaign: Pubkey,      // Kampania
    pub contributor: Pubkey,   // Uczestnik
    pub amount: u64,           // Kwota wpłaty
    pub total_raised: u64,     // Łączna zebrana kwota
}
```

### 💸 FundsWithdrawn
```rust
pub struct FundsWithdrawn {
    pub campaign: Pubkey,      // Kampania
    pub creator: Pubkey,       // Twórca
    pub amount: u64,           // Wypłacona kwota
}
```

### 🔄 ContributionRefunded
```rust
pub struct ContributionRefunded {
    pub campaign: Pubkey,      // Kampania  
    pub contributor: Pubkey,   // Uczestnik
    pub amount: u64,           // Zwrócona kwota
}
```

## ❌ Obsługa błędów

Kompletny system obsługi błędów z opisowymi komunikatami:

### ⚠️ Błędy walidacji:
- `TitleTooLong`: Tytuł przekracza 100 znaków
- `DescriptionTooLong`: Opis przekracza 500 znaków  
- `InvalidTargetAmount`: Nieprawidłowa kwota docelowa
- `InvalidDuration`: Czas trwania poza zakresem 1-365 dni

### 📅 Błędy stanu kampanii:
- `CampaignEnded`: Próba wpłaty po zakończeniu
- `CampaignStillActive`: Próba zwrotu w aktywnej kampanii
- `CampaignWasSuccessful`: Próba zwrotu z udanej kampanii
- `CampaignAlreadyWithdrawn`: Próba wpłaty po wypłacie środków

### 💰 Błędy finansowe:
- `InvalidContributionAmount`: Nieprawidłowa kwota wpłaty
- `ExceedsTarget`: Wpłata przekracza cel kampanii
- `AmountOverflow`: Przepełnienie arytmetyczne
- `NoFundsToWithdraw`: Brak środków do wypłaty
- `NoContributionToRefund`: Brak wkładu do zwrotu

### 🔐 Błędy autoryzacji:
- `UnauthorizedWithdrawal`: Nieautoryzowana próba wypłaty
- `WithdrawalConditionsNotMet`: Niespełnione warunki wypłaty
- `AlreadyWithdrawn`: Środki już wypłacone

## 💻 Użycie

### 🚀 Przykład przepływu:

1. **Tworzenie kampanii**:
   ```typescript
   await program.methods
     .initializeCampaign("Moja kampania", "Opis kampanii", new BN(1000000), 30)
     .accounts({ /* konta */ })
     .rpc();
   ```

2. **Wpłacanie środków**:
   ```typescript
   await program.methods
     .contribute(new BN(100000))
     .accounts({ /* konta */ })
     .rpc();
   ```

3. **Wypłata środków** (po sukcesie):
   ```typescript
   await program.methods
     .withdrawFunds()
     .accounts({ /* konta */ })
     .rpc();
   ```

4. **Zwrot środków** (po niepowodzeniu):
   ```typescript
   await program.methods
     .refundContribution()
     .accounts({ /* konta */ })
     .rpc();
   ```

## 🧪 Testowanie

### 📋 Scenariusze testowe:

#### ✅ Testy pozytywne:
- Tworzenie kampanii z prawidłowymi parametrami
- Wpłacanie środków w różnych kwotach
- Wypłata po osiągnięciu celu
- Wypłata po zakończeniu czasu (bez osiągnięcia celu)
- Zwrot środków z nieudanej kampanii

#### ❌ Testy negatywne:
- Próba tworzenia kampanii z nieprawidłowymi parametrami
- Wpłacanie po zakończeniu kampanii
- Próba wypłaty przez nieuprawnioną osobę
- Próba zwrotu z udanej kampanii
- Próba wielokrotnej wypłaty/zwrotu

#### 🔒 Testy bezpieczeństwa:
- Overflow attacks
- Próby obejścia autoryzacji
- Manipulacja stanu kampanii
- Testy graniczne parametrów

### 📁 Struktura testów:
```
tests/
├── integration/
│   ├── campaign_lifecycle.ts
│   ├── contribution_flow.ts
│   ├── withdrawal_scenarios.ts
│   └── refund_scenarios.ts
├── unit/
│   ├── validation.ts
│   ├── calculations.ts
│   └── error_handling.ts
└── security/
    ├── authorization.ts
    ├── overflow_protection.ts
    └── state_manipulation.ts
```

## 📈 Wnioski

Ten smart contract przedstawia kompleksowe rozwiązanie crowdfundingowe dla ekosystemu Solana, implementujące najlepsze praktyki bezpieczeństwa i architektury. Wykorzystuje zaawansowane funkcje Anchor framework'a i mechanizmy Solany do zapewnienia bezpiecznej, skalowalnej i efektywnej platformy crowdfundingowej.

### 🎯 Kluczowe osiągnięcia:
- **Bezpieczeństwo**: Komprehensywne mechanizmy ochrony
- **Przejrzystość**: Pełna auditabilność wszystkich operacji  
- **Efektywność**: Optymalne wykorzystanie zasobów Solany
- **Skalowalność**: Architektura umożliwiająca nieograniczony wzrost
- **Użyteczność**: Intuicyjne API dla programistów frontend

### 🚀 Możliwości rozwoju:
- Implementacja częściowych zwrotów
- System opłat platformy
- Mechanizmy stake'owania dla weryfikacji
- Integracja z Oracle'ami dla kursów walut
- System reputacji twórców kampanii
