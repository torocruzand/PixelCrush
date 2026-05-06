# Contributing to PixelCrush

First off, thank you for considering contributing to PixelCrush! It's people like you that make open-source software such a great community to learn, inspire, and create.

The following is a set of guidelines for contributing to PixelCrush. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Code of Conduct

This project and everyone participating in it is governed by the [PixelCrush Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to hello@torocruzand.com.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

* Use a clear and descriptive title for the issue to identify the problem.
* Describe the exact steps which reproduce the problem in as many details as possible.
* Provide specific examples to demonstrate the steps.
* Describe the behavior you observed after following the steps and point out what exactly is the problem with that behavior.
* Explain which behavior you expected to see instead and why.
* Include screenshots and animated GIFs if possible.

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include as many details as possible:

* Use a clear and descriptive title for the issue to identify the suggestion.
* Provide a step-by-step description of the suggested enhancement in as many details as possible.
* Provide specific examples to demonstrate the steps.
* Describe the current behavior and explain which behavior you expected to see instead and why.
* Explain why this enhancement would be useful to most PixelCrush users.

### Pull Requests

* Fill in the required template.
* Do not include issue numbers in the PR title.
* Follow the Rust style guide (`cargo fmt`).
* Include tests when adding new features or fixing bugs.
* Document new code based on the Documentation Style Guide.
* End files with a newline.

## Development Setup

1. **Rust:** You'll need the Rust compiler and Cargo. Install them from [rustup.rs](https://rustup.rs/).
2. **Clone the repository:**
   ```bash
   git clone https://github.com/torocruzand/PixelCrush.git
   cd PixelCrush
   ```
3. **Build and Run:**
   ```bash
   cargo run
   ```

## Styleguides

### Git Commit Messages

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line
