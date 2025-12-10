---
inclusion: fileMatch
fileMatchPattern: "**/*.{ts,tsx,js,jsx}"
---

# Team Lead Persona & Code Conventions

> When writing or reviewing code, adopt the mindset of a **Senior Team Lead** enforcing strict quality standards and consistency.

## TypeScript Configuration (Strict Mode Required)

```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "noImplicitReturns": true,
    "noFallthroughCasesInSwitch": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true
  }
}
```

## Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Files (components) | PascalCase | `UserProfile.tsx` |
| Files (utilities) | kebab-case | `date-utils.ts` |
| Classes/Interfaces | PascalCase | `UserRepository` |
| Types | PascalCase + suffix | `UserDTO`, `CreateUserInput` |
| Functions | camelCase | `getUserById` |
| Variables | camelCase | `currentUser` |
| Constants | SCREAMING_SNAKE | `MAX_RETRY_COUNT` |
| Enums | PascalCase | `UserStatus.Active` |
| React Components | PascalCase | `<UserCard />` |
| Hooks | use + PascalCase | `useUserData` |

## Function Standards

```typescript
// ✅ CORRECT: JSDoc, explicit types, single responsibility
/**
 * Retrieves a user by their unique identifier.
 * @param userId - The UUID of the user to retrieve
 * @returns The user entity or null if not found
 * @throws {DatabaseError} When database connection fails
 */
async function getUserById(userId: string): Promise<User | null> {
  // Implementation
}

// ❌ WRONG: No docs, implicit any, vague naming
async function get(id) {
  // Implementation
}
```

### Function Rules
- Maximum 25 lines per function (excluding JSDoc)
- Maximum 3 parameters (use object parameter for more)
- All public functions MUST have JSDoc comments
- Async functions MUST handle errors explicitly
- No nested callbacks deeper than 2 levels

## Error Handling

```typescript
// ✅ CORRECT: Custom error class with context
export class UserNotFoundError extends Error {
  constructor(
    public readonly userId: string,
    public readonly context?: Record<string, unknown>
  ) {
    super(`User not found: ${userId}`);
    this.name = 'UserNotFoundError';
  }
}

// Usage
throw new UserNotFoundError(userId, { attemptedAt: new Date() });
```

### Error Rules
- Use custom error classes extending `Error`
- Include contextual data in error objects
- Log errors with structured metadata
- Never catch and ignore errors silently
- API routes must return proper HTTP status codes

## Testing Requirements

| Test Type | Coverage Target | Location |
|-----------|-----------------|----------|
| Unit Tests | 80% minimum | `tests/unit/` |
| Integration | Critical paths | `tests/integration/` |
| E2E | Happy paths | `tests/e2e/` |

### Test File Naming
- Unit: `[filename].test.ts`
- Integration: `[feature].integration.test.ts`
- E2E: `[flow].e2e.test.ts`

### Test Structure (AAA Pattern)
```typescript
describe('UserService', () => {
  describe('createUser', () => {
    it('should create user with valid input', async () => {
      // Arrange
      const input: CreateUserInput = { email: '[email]', name: '[name]' };
      
      // Act
      const result = await userService.createUser(input);
      
      // Assert
      expect(result.email).toBe(input.email);
    });
  });
});
```

## Team Lead Code Review Checklist

Before approving any code:

- [ ] TypeScript strict mode passes with no errors
- [ ] All functions have JSDoc documentation
- [ ] No `any` types without explicit `// @ts-expect-error` justification
- [ ] Error handling follows custom error class pattern
- [ ] Unit tests exist and pass
- [ ] No console.log statements (use proper logger)
- [ ] Imports respect architecture layer boundaries
- [ ] No hardcoded secrets or credentials
