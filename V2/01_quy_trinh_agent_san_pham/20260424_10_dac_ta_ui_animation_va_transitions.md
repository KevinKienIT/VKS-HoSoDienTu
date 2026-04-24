# DAC TA UI ANIMATION VA TRANSITIONS

Ngay tao: 2026-04-24
Ngay cap nhat gan nhat: 2026-04-24
Sua boi agent: Codex
Muc dich: Dinh nghia cac hieu ung chuyen canh, animation, va transition cho ung dung VKS ECMS.
Nguon prompt: Tu yeu cau nguoi dung ve hieu ung chuyen canh muot ma, giao dien dep nhu web.

---

## 1. Animation Strategy Overview

### 1.1 Thumbnail Luu Dung

| Animation Type | Thu Vien Ho Tro |
|--------------|---------------|
| Component entrance/exit | Framer Motion |
| Page transitions | Framer Motion |
| List stagger | Framer Motion |
| Gesture animations | Framer Motion |
| Layout transitions | Framer Motion |
| Modal/Dialog | React Transition Group |
| Toast notifications | Framer Motion |

### 1.2 Animation Constants

```typescript
// /src/constants/animations.ts
export const ANIMATION = {
  // Durations (in ms)
  duration: {
    fast: 150,
    normal: 200,
    slow: 300,
    page: 400
  },
  
  // Ease curves
  ease: {
    standard: [0.4, 0, 0.2, 1],  // ease-out
    emphasized: [0.4, 0, 0.2, 1],
    decelerate: [0.4, 0, 0.2, 1],
    accelerate: [0.4, 0, 0.2, 1]
  },
  
  // Spring configs
  spring: {
    gentle: { stiffness: 120, damping: 14 },
    default: { stiffness: 200, damping: 20 },
    emphasized: { stiffness: 300, damping: 24 }
  }
};
```

---

## 2. Page Transitions

### 2.1 Route Change Animation

```typescript
// Page transition variants
export const pageVariants = {
  initial: {
    opacity: 0,
    y: 8,
    scale: 0.98
  },
  animate: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: {
      duration: 0.4,
      ease: [0.4, 0, 0.2, 1]
    }
  },
  exit: {
    opacity: 0,
    y: -8,
    scale: 0.98,
    transition: {
      duration: 0.2,
      ease: [0.4, 0, 0.2, 1]
    }
  }
};
```

### 2.2 Tab Switch Animation

```typescript
const tabVariants = {
  enter: {
    opacity: 0,
    x: 20
  },
  center: {
    opacity: 1,
    x: 0,
    transition: {
      duration: 0.2,
      ease: 'easeOut'
    }
  },
  exit: {
    opacity: 0,
    x: -20,
    transition: {
      duration: 0.15,
      ease: 'easeIn'
    }
  }
};
```

### 2.3 Side Panel Overlay

```typescript
const panelVariants = {
  hidden: {
    x: '100%',
    opacity: 0
  },
  visible: {
    x: 0,
    opacity: 1,
    transition: {
      type: 'spring',
      stiffness: 300,
      damping: 30
    }
  },
  exit: {
    x: '100%',
    opacity: 0,
    transition: {
      duration: 0.2,
      ease: 'easeIn'
    }
  }
};
```

---

## 3. List Animations

### 3.1 Case List / Document List

```typescript
// Stagger container
const listContainer = {
  hidden: { opacity: 0 },
  show: {
    opacity: 1,
    transition: {
      staggerChildren: 0.05,
      delayChildren: 0.1
    }
  }
};

// Individual item
const listItem = {
  hidden: {
    opacity: 0,
    y: 10
  },
  show: {
    opacity: 1,
    y: 0,
    transition: {
      type: 'spring',
      stiffness: 200,
      damping: 20
    }
  }
};

// Usage in component
<motion.ul
  variants={listContainer}
  initial="hidden"
  animate="show"
>
  {items.map(item => (
    <motion.li key={item.id} variants={listItem}>
      <CaseCard case={item} />
    </motion.li>
  ))}
</motion.ul>
```

### 3.2 Search Results Animation

```typescript
// Different stagger for search results (faster)
const searchResultVariants = {
  hidden: {
    opacity: 0,
    y: 10,
    scale: 0.98
  },
  show: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: {
      staggerChildren: 0.03
    }
  }
};
```

### 3.3 Skeleton Loading

```typescript
const skeletonVariants = {
  initial: {
    opacity: 0.5
  },
  animate: {
    opacity: 1,
    transition: {
      duration: 0.8,
      repeat: Infinity,
      repeatType: 'reverse'
    }
  }
};

// Skeleton card
<motion.div
  variants={skeletonVariants}
  initial="initial"
  animate="animate"
  style={{ backgroundColor: '#e0e0e0' }}
/>
```

---

## 4. Viewer Animations

### 4.1 Document Open Animation

```typescript
const documentOpenVariants = {
  initial: {
    opacity: 0,
    scale: 0.95
  },
  animate: {
    opacity: 1,
    scale: 1,
    transition: {
      duration: 0.3,
      ease: [0.4, 0, 0.2, 1]
    }
  }
};
```

### 4.2 Zoom Animation

```typescript
// Smooth zoom with spring
const zoomVariants = {
  small: { scale: 0.5 },
  medium: { scale: 1 },
  large: { scale: 1.5 },
  fitWidth: { scale: 'fit-width' },
  fitPage: { scale: 'fit-page' }
};

// Zoom gesture
<motion.div
  drag
  dragConstraints={{ left: 0, right: 0, top: 0, bottom: 0 }}
  onDragEnd={(e, info) => {
    // Handle pan
  }}
/>
```

### 4.3 Page Turn Animation

```typescript
const pageTurnVariants = {
  enter: (direction: number) => ({
    x: direction > 0 ? '100%' : '-100%',
    opacity: 0
  }),
  center: {
    x: 0,
    opacity: 1,
    transition: {
      type: 'spring',
      stiffness: 200,
      damping: 25
    }
  },
  exit: (direction: number) => ({
    x: direction < 0 ? '100%' : '-100%',
    opacity: 0,
    transition: {
      type: 'spring',
      stiffness: 200,
      damping: 25
    }
  })
};
```

---

## 5. Interactive Elements

### 5.1 Button Interactions

```typescript
const buttonVariants = {
  idle: {
    scale: 1,
    boxShadow: '0px 0px 0px rgba(0,0,0,0)'
  },
  hover: {
    scale: 1.02,
    boxShadow: '0px 2px 4px rgba(0,0,0,0.1)',
    transition: {
      duration: 0.15
    }
  },
  tap: {
    scale: 0.98,
    boxShadow: '0px 0px 0px rgba(0,0,0,0)',
    transition: {
      duration: 0.1
    }
  },
  focus: {
    scale: 1,
    boxShadow: '0 0 0 2px #2196F3'
  }
};
```

### 5.2 Card Hover Effect

```typescript
const cardHoverVariants = {
  rest: {
    y: 0,
    boxShadow: '0px 1px 3px rgba(0,0,0,0.12)'
  },
  hover: {
    y: -4,
    boxShadow: '0px 4px 12px rgba(0,0,0,0.15)',
    transition: {
      type: 'spring',
      stiffness: 300,
      damping: 20
    }
  }
};
```

### 5.3 Input Focus

```typescript
const inputVariants = {
  idle: {
    borderColor: '#e0e0e0',
    boxShadow: 'none'
  },
  focus: {
    borderColor: '#2196F3',
    boxShadow: '0 0 0 3px rgba(33,150,243,0.1)',
    transition: {
      duration: 0.2
    }
  }
};
```

---

## 6. Modal & Dialog

### 6.1 Modal Animation

```typescript
const modalVariants = {
  hidden: {
    opacity: 0,
    scale: 0.9
  },
  visible: {
    opacity: 1,
    scale: 1,
    transition: {
      type: 'spring',
      stiffness: 300,
      damping: 25
    }
  },
  exit: {
    opacity: 0,
    scale: 0.9,
    transition: {
      duration: 0.15
    }
  }
};

const overlayVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      duration: 0.2
    }
  },
  exit: { opacity: 0 }
};
```

### 6.2 Toast Notification

```typescript
const toastVariants = {
  hidden: {
    y: 50,
    opacity: 0
  },
  visible: {
    y: 0,
    opacity: 1,
    transition: {
      type: 'spring',
      stiffness: 300,
      damping: 25
    }
  },
  exit: {
    y: 50,
    opacity: 0,
    transition: {
      duration: 0.2
    }
  }
};
```

---

## 7. Responsive Animations

### 7.1 Mobile Drawer

```typescript
const drawerVariants = {
  closed: {
    x: '-100%',
    transition: {
      type: 'spring',
      stiffness: 300,
      damping: 30
    }
  },
  open: {
    x: 0,
    transition: {
      type: 'spring',
      stiffness: 300,
      damping: 30
    }
  }
};
```

### 7.2 Collapse/Expand

```typescript
const collapseVariants = {
  collapsed: {
    height: 0,
    opacity: 0,
    transition: {
      duration: 0.2
    }
  },
  expanded: {
    height: 'auto',
    opacity: 1,
    transition: {
      duration: 0.3,
      ease: [0.4, 0, 0.2, 1]
    }
  }
};
```

---

## 8. Performance Guidelines

### 8.1 Optimization Rules

| Rule | Implementation |
|------|---------------|
| Use `transform` | Animate `x, y, scale` not `top, left` |
| Use opacity sparingly | Keep opacity last in animation |
| Limit animated props | Max 4 props per animation |
| Use `will-change` | For complex animations |
| Use `layout` prop | For layout changes |

### 8.2 Loading States

- Show skeleton before content loads
- Stagger loading for lists > 10 items
- Use `layout` animation for list reorder

### 8.3 Accessibility

- Respect `prefers-reduced-motion`
- Provide alternative for complex animations
- Ensure focus visible during transitions

---

## 9. Acceptance Criteria

- [ ] Page transitions sot mau voi Framer Motion
- [ ] List items co stagger animation
- [ ] Viewer co zoom/pan muot
- [ ] Button/card co hover/tap effects
- [ ] Modal co fade + scale animation
- [ ] Responsive tren mobile
- [ ] Ho tro prefers-reduced-motion

---

**STATUS: SPECIFICATION DEFINED. WAITING FOR IMPLEMENTATION.**