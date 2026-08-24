# Hblank

Hblank is the context for developing native UI components in isolation, cataloging their meaningful states, and exercising those states through one local workbench.

## Language

**Component**:
A cataloged UI renderer with one presentation-props schema and shared documentation.
_Avoid_: Story, fixture

**Fixture variant**:
A named, discoverable default state of one component.
_Avoid_: Story, example, component

**Fixture file**:
A discovered project source file that may define components and their fixture variants.
_Avoid_: Story file, example file

**Canonical fixture id**:
The project-relative source path and fixture function symbol that identify exactly one fixture variant.
_Avoid_: Alias, slug

**Catalog**:
The component-first hierarchy of groups, components, fixture variants, controls, and documentation.
_Avoid_: Story tree

**Harness**:
The native application surface in which the catalog is browsed and components are rendered, controlled, documented, and tested.
_Avoid_: Storybook

