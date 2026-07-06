name: Pull Request Template
description: Template for pull requests
title: "[PR]: "
labels: []
body:
  - type: textarea
    attributes:
      label: Description
      description: What does this PR change?
    validations:
      required: true
  - type: checkboxes
    attributes:
      label: Self-check
      options:
        - label: I have tested these changes
        - label: I have updated documentation if needed
        - label: Code follows project style
  - type: textarea
    attributes:
      label: Related Issues
      description: Links to related issues
