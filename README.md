# reminder
Command-line reminder app written in Rust

This Rust program implements a spaced repetition reminder system with the following features:
Installation and Usage

### Build and install:
```cargo build --release```
### Optionally install globally
```cargo install --path .```

### Usage Examples

Add a new reminder:
```reminder add "Learn Rust ownership concepts"```

Check for due reminders:
```reminder check```

List all reminders:
```reminder list```

Mark a reminder as reviewed:
```reminder review 1```

Remove a reminder:
```reminder remove 1```


### How the Spaced Repetition Works

Initial reminder: Added to review queue, first review due in 1 day
- After 1st review: Next review in 3 days
- After 2nd review: Next review in 1 week
- After 3rd review: Next review in 1 month
- After 4th review: Reminder marked as completed
