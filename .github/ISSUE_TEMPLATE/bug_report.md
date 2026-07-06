name: Bug Report
description: Report a bug in RunST X kernel
title: "[BUG]: "
labels: ["bug"]
body:
  - type: textarea
    attributes:
      label: Describe the bug
      description: A clear and concise description of what the bug is.
    validations:
      required: true
  - type: textarea
    attributes:
      label: Steps to Reproduce
      description: Steps to reproduce the behavior.
      value: |
        1. Boot the kernel
        2. Run command '...'
        3. Observe error '...'
    validations:
      required: true
  - type: textarea
    attributes:
      label: Expected behavior
      description: What you expected to happen.
    validations:
      required: true
  - type: textarea
    attributes:
      label: Actual behavior
      description: What actually happened.
    validations:
      required: true
  - type: textarea
    attributes:
      label: Kernel Version
      description: What version of RunST X are you running?
      value: "v0.4"
  - type: textarea
    attributes:
      label: Additional context
      description: Add any other context about the problem here.
