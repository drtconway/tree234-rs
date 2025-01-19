use std::collections::VecDeque;

use crate::either::Either;

pub type Item<K, V> = (K, V);

enum Node<K: Eq + Ord, V> {
    Empty,
    Two(Two<K, V>),
    Three(Three<K, V>),
    Four(Four<K, V>),
}

impl<K: Eq + Ord, V> Node<K, V> {
    fn empty() -> NodeBox<K, V> {
        Box::new(Node::Empty)
    }

    fn two(item: Item<K, V>, lhs: NodeBox<K, V>, rhs: NodeBox<K, V>) -> NodeBox<K, V> {
        let size = 1 + lhs.size() + rhs.size();
        Box::new(Node::Two(Two {
            size,
            item,
            lhs,
            rhs,
        }))
    }

    fn three(
        item1: Item<K, V>,
        item2: Item<K, V>,
        lhs: NodeBox<K, V>,
        mid: NodeBox<K, V>,
        rhs: NodeBox<K, V>,
    ) -> NodeBox<K, V> {
        let size = 2 + lhs.size() + mid.size() + rhs.size();
        Box::new(Node::Three(Three {
            size,
            item1,
            item2,
            lhs,
            mid,
            rhs,
        }))
    }

    fn four(
        item1: Item<K, V>,
        item2: Item<K, V>,
        item3: Item<K, V>,
        lhs: NodeBox<K, V>,
        lhs_mid: NodeBox<K, V>,
        rhs_mid: NodeBox<K, V>,
        rhs: NodeBox<K, V>,
    ) -> NodeBox<K, V> {
        let size = 3 + lhs.size() + lhs_mid.size() + rhs_mid.size() + rhs.size();
        Box::new(Node::Four(Four {
            size,
            item1,
            item2,
            item3,
            lhs,
            lhs_mid,
            rhs_mid,
            rhs,
        }))
    }

    fn is_empty(&self) -> bool {
        match self {
            Node::Empty => true,
            _ => false,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Node::Empty => 0,
            Node::Two(two) => two.size,
            Node::Three(three) => three.size,
            Node::Four(four) => four.size,
        }
    }

    pub fn get(&self, key: &K) -> Option<&Item<K, V>> {
        match self {
            Node::Empty => None,
            Node::Two(two) => match key.cmp(&two.item.0) {
                std::cmp::Ordering::Less => two.lhs.get(key),
                std::cmp::Ordering::Equal => Some(&two.item),
                std::cmp::Ordering::Greater => two.rhs.get(key),
            },
            Node::Three(three) => match key.cmp(&three.item1.0) {
                std::cmp::Ordering::Less => three.lhs.get(key),
                std::cmp::Ordering::Equal => Some(&three.item1),
                std::cmp::Ordering::Greater => match key.cmp(&three.item2.0) {
                    std::cmp::Ordering::Less => three.mid.get(key),
                    std::cmp::Ordering::Equal => Some(&three.item2),
                    std::cmp::Ordering::Greater => three.rhs.get(key),
                },
            },
            Node::Four(four) => match key.cmp(&four.item1.0) {
                std::cmp::Ordering::Less => four.lhs.get(key),
                std::cmp::Ordering::Equal => Some(&four.item1),
                std::cmp::Ordering::Greater => match key.cmp(&four.item2.0) {
                    std::cmp::Ordering::Less => four.lhs_mid.get(key),
                    std::cmp::Ordering::Equal => Some(&four.item2),
                    std::cmp::Ordering::Greater => match key.cmp(&four.item3.0) {
                        std::cmp::Ordering::Less => four.rhs_mid.get(key),
                        std::cmp::Ordering::Equal => Some(&four.item3),
                        std::cmp::Ordering::Greater => four.rhs.get(key),
                    },
                },
            },
        }
    }

    pub fn insert(self, key: K, value: V) -> NodeBox<K, V> {
        match self {
            Node::Empty => Node::two((key, value), Node::empty(), Node::empty()),
            Node::Two(two) => Node::insert2(two, key, value),
            Node::Three(three) => Node::insert3(three, key, value),
            Node::Four(four) => {
                let (mid_item, mid_lhs, mid_rhs) = Node::split_four(four);
                match key.cmp(&mid_item.0) {
                    std::cmp::Ordering::Less => {
                        let mid_lhs = mid_lhs.insert(key, value);
                        Node::two(mid_item, mid_lhs, mid_rhs)
                    }
                    std::cmp::Ordering::Equal => {
                        let mid_item = (mid_item.0, value);
                        Node::two(mid_item, mid_lhs, mid_rhs)
                    }
                    std::cmp::Ordering::Greater => {
                        let mid_rhs = mid_rhs.insert(key, value);
                        Node::two(mid_item, mid_lhs, mid_rhs)
                    }
                }
            }
        }
    }

    fn insert2(two: Two<K, V>, key: K, value: V) -> NodeBox<K, V> {
        let Two {
            size: _,
            item,
            lhs,
            rhs,
        } = two;
        if lhs.is_empty() {
            assert!(rhs.is_empty());
            match key.cmp(&item.0) {
                std::cmp::Ordering::Less => {
                    Node::three((key, value), item, lhs, Node::empty(), rhs)
                }
                std::cmp::Ordering::Equal => Node::two((item.0, value), lhs, rhs),
                std::cmp::Ordering::Greater => {
                    Node::three(item, (key, value), lhs, Node::empty(), rhs)
                }
            }
        } else {
            match key.cmp(&item.0) {
                std::cmp::Ordering::Less => match *lhs {
                    Node::Empty => {
                        let lhs = Node::two((key, value), Node::empty(), Node::empty());
                        Node::two(item, lhs, rhs)
                    }
                    Node::Two(two) => {
                        let lhs = Node::insert2(two, key, value);
                        Node::two(item, lhs, rhs)
                    }
                    Node::Three(three) => {
                        let lhs = Node::insert3(three, key, value);
                        Node::two(item, lhs, rhs)
                    }
                    Node::Four(four) => {
                        let (mid_item, mid_lhs, mid_rhs) = Node::split_four(four);
                        match key.cmp(&mid_item.0) {
                            std::cmp::Ordering::Less => {
                                let mid_lhs = mid_lhs.insert(key, value);
                                Node::three(mid_item, item, mid_lhs, mid_rhs, rhs)
                            }
                            std::cmp::Ordering::Equal => {
                                let mid_item = (mid_item.0, value);
                                Node::three(mid_item, item, mid_lhs, mid_rhs, rhs)
                            }
                            std::cmp::Ordering::Greater => {
                                let mid_rhs = mid_rhs.insert(key, value);
                                Node::three(mid_item, item, mid_lhs, mid_rhs, rhs)
                            }
                        }
                    }
                },
                std::cmp::Ordering::Equal => {
                    let item = (item.0, value);
                    Node::two(item, lhs, rhs)
                }
                std::cmp::Ordering::Greater => match *rhs {
                    Node::Empty => {
                        let rhs = Node::two((key, value), Node::empty(), Node::empty());
                        Node::two(item, lhs, rhs)
                    }
                    Node::Two(two) => {
                        let rhs = Node::insert2(two, key, value);
                        Node::two(item, lhs, rhs)
                    }
                    Node::Three(three) => {
                        let rhs = Node::insert3(three, key, value);
                        Node::two(item, lhs, rhs)
                    }
                    Node::Four(four) => {
                        let (mid_item, mid_lhs, mid_rhs) = Node::split_four(four);
                        match key.cmp(&mid_item.0) {
                            std::cmp::Ordering::Less => {
                                let mid_lhs = mid_lhs.insert(key, value);
                                Node::three(item, mid_item, lhs, mid_lhs, mid_rhs)
                            }
                            std::cmp::Ordering::Equal => {
                                let mid_item = (mid_item.0, value);
                                Node::three(item, mid_item, lhs, mid_lhs, mid_rhs)
                            }
                            std::cmp::Ordering::Greater => {
                                let mid_rhs = mid_rhs.insert(key, value);
                                Node::three(item, mid_item, lhs, mid_lhs, mid_rhs)
                            }
                        }
                    }
                },
            }
        }
    }

    fn insert3(three: Three<K, V>, key: K, value: V) -> NodeBox<K, V> {
        let Three {
            size: _,
            item1,
            item2,
            lhs,
            mid,
            rhs,
        } = three;
        if lhs.is_empty() {
            assert!(mid.is_empty());
            assert!(rhs.is_empty());
            match key.cmp(&item1.0) {
                std::cmp::Ordering::Less => {
                    Node::four((key, value), item1, item2, lhs, mid, Node::empty(), rhs)
                }
                std::cmp::Ordering::Equal => Node::three((item1.0, value), item2, lhs, mid, rhs),
                std::cmp::Ordering::Greater => match key.cmp(&item2.0) {
                    std::cmp::Ordering::Less => {
                        Node::four(item1, (key, value), item2, lhs, mid, Node::empty(), rhs)
                    }
                    std::cmp::Ordering::Equal => {
                        Node::three(item1, (item2.0, value), lhs, mid, rhs)
                    }
                    std::cmp::Ordering::Greater => {
                        Node::four(item1, item2, (key, value), lhs, mid, Node::empty(), rhs)
                    }
                },
            }
        } else {
            match key.cmp(&item1.0) {
                std::cmp::Ordering::Less => match *lhs {
                    Node::Empty => {
                        let lhs = Node::two((key, value), Node::empty(), Node::empty());
                        Node::three(item1, item2, lhs, mid, rhs)
                    }
                    Node::Two(two) => {
                        let lhs = Node::insert2(two, key, value);
                        Node::three(item1, item2, lhs, mid, rhs)
                    }
                    Node::Three(three) => {
                        let lhs = Node::insert3(three, key, value);
                        Node::three(item1, item2, lhs, mid, rhs)
                    }
                    Node::Four(four) => {
                        let (mid_item, mid_lhs, mid_rhs) = Node::split_four(four);
                        match key.cmp(&mid_item.0) {
                            std::cmp::Ordering::Less => {
                                let mid_lhs = mid_lhs.insert(key, value);
                                Node::four(mid_item, item1, item2, mid_lhs, mid_rhs, mid, rhs)
                            }
                            std::cmp::Ordering::Equal => {
                                let mid_item = (mid_item.0, value);
                                Node::four(mid_item, item1, item2, mid_lhs, mid_rhs, mid, rhs)
                            }
                            std::cmp::Ordering::Greater => {
                                let mid_rhs = mid_rhs.insert(key, value);
                                Node::four(mid_item, item1, item2, mid_lhs, mid_rhs, mid, rhs)
                            }
                        }
                    }
                },
                std::cmp::Ordering::Equal => Node::three((item1.0, value), item2, lhs, mid, rhs),
                std::cmp::Ordering::Greater => match key.cmp(&item2.0) {
                    std::cmp::Ordering::Less => match *mid {
                        Node::Empty => {
                            let mid = Node::two((key, value), Node::empty(), Node::empty());
                            Node::three(item1, item2, lhs, mid, rhs)
                        }
                        Node::Two(two) => {
                            let mid = Node::insert2(two, key, value);
                            Node::three(item1, item2, lhs, mid, rhs)
                        }
                        Node::Three(three) => {
                            let mid = Node::insert3(three, key, value);
                            Node::three(item1, item2, lhs, mid, rhs)
                        }
                        Node::Four(four) => {
                            let (mid_item, mid_lhs, mid_rhs) = Node::split_four(four);
                            match key.cmp(&mid_item.0) {
                                std::cmp::Ordering::Less => {
                                    let mid_lhs = mid_lhs.insert(key, value);
                                    Node::four(item1, mid_item, item2, lhs, mid_lhs, mid_rhs, rhs)
                                }
                                std::cmp::Ordering::Equal => {
                                    let mid_item = (mid_item.0, value);
                                    Node::four(item1, mid_item, item2, lhs, mid_lhs, mid_rhs, rhs)
                                }
                                std::cmp::Ordering::Greater => {
                                    let mid_rhs = mid_rhs.insert(key, value);
                                    Node::four(item1, mid_item, item2, lhs, mid_lhs, mid_rhs, rhs)
                                }
                            }
                        }
                    },
                    std::cmp::Ordering::Equal => {
                        Node::three(item1, (item2.0, value), lhs, mid, rhs)
                    }
                    std::cmp::Ordering::Greater => match *rhs {
                        Node::Empty => {
                            let rhs = Node::two((key, value), Node::empty(), Node::empty());
                            Node::three(item1, item2, lhs, mid, rhs)
                        }
                        Node::Two(two) => {
                            let rhs = Node::insert2(two, key, value);
                            Node::three(item1, item2, lhs, mid, rhs)
                        }
                        Node::Three(three) => {
                            let rhs = Node::insert3(three, key, value);
                            Node::three(item1, item2, lhs, mid, rhs)
                        }
                        Node::Four(four) => {
                            let (mid_item, mid_lhs, mid_rhs) = Node::split_four(four);
                            match key.cmp(&mid_item.0) {
                                std::cmp::Ordering::Less => {
                                    let mid_lhs = mid_lhs.insert(key, value);
                                    Node::four(item1, item2, mid_item, lhs, mid, mid_lhs, mid_rhs)
                                }
                                std::cmp::Ordering::Equal => {
                                    let mid_item = (mid_item.0, value);
                                    Node::four(item1, item2, mid_item, lhs, mid, mid_lhs, mid_rhs)
                                }
                                std::cmp::Ordering::Greater => {
                                    let mid_rhs = mid_rhs.insert(key, value);
                                    Node::four(item1, item2, mid_item, lhs, mid, mid_lhs, mid_rhs)
                                }
                            }
                        }
                    },
                },
            }
        }
    }

    fn split_four(four: Four<K, V>) -> (Item<K, V>, NodeBox<K, V>, NodeBox<K, V>) {
        let Four {
            size: _,
            item1,
            item2,
            item3,
            lhs,
            lhs_mid,
            rhs_mid,
            rhs,
        } = four;
        (
            item2,
            Node::two(item1, lhs, lhs_mid),
            Node::two(item3, rhs_mid, rhs),
        )
    }

    fn remove(self, key: &K) -> (NodeBox<K, V>, Option<V>, bool) {
        match self {
            Node::Empty => (Node::empty(), None, false),
            Node::Two(two) => Node::remove2(two, key),
            Node::Three(three) => Node::remove3(three, key),
            Node::Four(four) => Node::remove4(four, key),
        }
    }

    fn remove2(two: Two<K, V>, key: &K) -> (NodeBox<K, V>, Option<V>, bool) {
        let Two {
            size: _,
            item,
            lhs,
            rhs,
        } = two;
        match key.cmp(&item.0) {
            std::cmp::Ordering::Less => {
                let (lhs, result, reduced) = Node::remove(*lhs, key);
                match reduced {
                    true => Node::fix2_lhs(item, lhs, rhs, result),
                    false => (Node::two(item, lhs, rhs), result, false),
                }
            }
            std::cmp::Ordering::Equal => {
                if let Some((small, rhs, reduced)) = Node::remove_smallest(*rhs) {
                    match reduced {
                        true => Node::fix2_rhs(small, lhs, rhs, Some(item.1)),
                        false => (Node::two(small, lhs, rhs), Some(item.1), false),
                    }
                } else {
                    (lhs, Some(item.1), true)
                }
            }
            std::cmp::Ordering::Greater => {
                let (rhs, result, reduced) = Node::remove(*rhs, key);
                match reduced {
                    true => Node::fix2_rhs(item, lhs, rhs, result),
                    false => (Node::two(item, lhs, rhs), result, false),
                }
            }
        }
    }

    fn remove3(three: Three<K, V>, key: &K) -> (NodeBox<K, V>, Option<V>, bool) {
        let Three {
            size: _,
            item1,
            item2,
            lhs,
            mid,
            rhs,
        } = three;
        match key.cmp(&item1.0) {
            std::cmp::Ordering::Less => {
                let (lhs, result, reduced) = Node::remove(*lhs, key);
                match reduced {
                    true => Node::fix3_lhs(item1, item2, lhs, mid, rhs, result),
                    false => (Node::three(item1, item2, lhs, mid, rhs), result, false),
                }
            }
            std::cmp::Ordering::Equal => {
                if let Some((small, mid, reduced)) = Node::remove_smallest(*mid) {
                    match reduced {
                        true => Node::fix3_mid(small, item2, lhs, mid, rhs, Some(item1.1)),
                        false => (
                            Node::three(small, item2, lhs, mid, rhs),
                            Some(item1.1),
                            false,
                        ),
                    }
                } else {
                    (Node::two(item2, lhs, rhs), Some(item1.1), false)
                }
            }
            std::cmp::Ordering::Greater => match key.cmp(&item2.0) {
                std::cmp::Ordering::Less => {
                    let (mid, result, reduced) = Node::remove(*mid, key);
                    match reduced {
                        true => Node::fix3_mid(item1, item2, lhs, mid, rhs, result),
                        false => (Node::three(item1, item2, lhs, mid, rhs), result, false),
                    }
                }
                std::cmp::Ordering::Equal => {
                    if let Some((small, rhs, reduced)) = Node::remove_smallest(*rhs) {
                        match reduced {
                            true => Node::fix3_rhs(item1, small, lhs, mid, rhs, Some(item2.1)),
                            false => (
                                Node::three(item1, small, lhs, mid, rhs),
                                Some(item2.1),
                                false,
                            ),
                        }
                    } else {
                        (Node::two(item1, lhs, mid), Some(item2.1), false)
                    }
                }
                std::cmp::Ordering::Greater => {
                    let (rhs, result, reduced) = Node::remove(*rhs, key);
                    match reduced {
                        true => Node::fix3_rhs(item1, item2, lhs, mid, rhs, result),
                        false => (Node::three(item1, item2, lhs, mid, rhs), result, false),
                    }
                }
            },
        }
    }

    fn remove4(four: Four<K, V>, key: &K) -> (NodeBox<K, V>, Option<V>, bool) {
        let Four {
            size: _,
            item1,
            item2,
            item3,
            lhs,
            lhs_mid,
            rhs_mid,
            rhs,
        } = four;
        match key.cmp(&item2.0) {
            std::cmp::Ordering::Less => match key.cmp(&item1.0) {
                std::cmp::Ordering::Less => {
                    let (lhs, result, reduced) = Node::remove(*lhs, key);
                    match reduced {
                        true => {
                            Node::fix4_lhs(item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs, result)
                        }
                        false => (
                            Node::four(item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs),
                            result,
                            false,
                        ),
                    }
                }
                std::cmp::Ordering::Equal => {
                    if let Some((small, lhs_mid, reduced)) = Node::remove_smallest(*lhs_mid) {
                        match reduced {
                            true => Node::fix4_lhs_mid(
                                small,
                                item2,
                                item3,
                                lhs,
                                lhs_mid,
                                rhs_mid,
                                rhs,
                                Some(item1.1),
                            ),
                            false => (
                                Node::four(small, item2, item3, lhs, lhs_mid, rhs_mid, rhs),
                                Some(item1.1),
                                false,
                            ),
                        }
                    } else {
                        (
                            Node::three(item2, item3, lhs, rhs_mid, rhs),
                            Some(item1.1),
                            false,
                        )
                    }
                }
                std::cmp::Ordering::Greater => {
                    let (lhs_mid, result, reduced) = Node::remove(*lhs_mid, key);
                    match reduced {
                        true => Node::fix4_lhs_mid(
                            item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs, result,
                        ),
                        false => (
                            Node::four(item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs),
                            result,
                            false,
                        ),
                    }
                }
            },
            std::cmp::Ordering::Equal => {
                if let Some((small, rhs_mid, reduced)) = Node::remove_smallest(*rhs_mid) {
                    match reduced {
                        true => Node::fix4_rhs_mid(
                            item1,
                            small,
                            item3,
                            lhs,
                            lhs_mid,
                            rhs_mid,
                            rhs,
                            Some(item2.1),
                        ),
                        false => (
                            Node::four(item1, small, item3, lhs, lhs_mid, rhs_mid, rhs),
                            Some(item2.1),
                            false,
                        ),
                    }
                } else {
                    (
                        Node::three(item1, item3, lhs, lhs_mid, rhs),
                        Some(item2.1),
                        false,
                    )
                }
            }
            std::cmp::Ordering::Greater => match key.cmp(&item3.0) {
                std::cmp::Ordering::Less => {
                    let (rhs_mid, result, reduced) = Node::remove(*rhs_mid, key);
                    match reduced {
                        true => Node::fix4_rhs_mid(
                            item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs, result,
                        ),
                        false => (
                            Node::four(item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs),
                            result,
                            false,
                        ),
                    }
                }
                std::cmp::Ordering::Equal => {
                    if let Some((small, rhs, reduced)) = Node::remove_smallest(*rhs) {
                        match reduced {
                            true => Node::fix4_rhs(
                                item1,
                                item2,
                                small,
                                lhs,
                                lhs_mid,
                                rhs_mid,
                                rhs,
                                Some(item3.1),
                            ),
                            false => (
                                Node::four(item1, item2, small, lhs, lhs_mid, rhs_mid, rhs),
                                Some(item3.1),
                                false,
                            ),
                        }
                    } else {
                        (
                            Node::three(item1, item2, lhs, lhs_mid, rhs_mid),
                            Some(item3.1),
                            false,
                        )
                    }
                }
                std::cmp::Ordering::Greater => {
                    let (rhs, result, reduced) = Node::remove(*rhs, key);
                    match reduced {
                        true => {
                            Node::fix4_rhs(item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs, result)
                        }
                        false => (
                            Node::four(item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs),
                            result,
                            false,
                        ),
                    }
                }
            },
        }
    }

    fn remove_smallest(self: Node<K, V>) -> Option<(Item<K, V>, NodeBox<K, V>, bool)> {
        match self {
            Node::Empty => None,
            Node::Two(two) => Node::remove_smallest2(two),
            Node::Three(three) => Node::remove_smallest3(three),
            Node::Four(four) => Node::remove_smallest4(four),
        }
    }

    fn remove_smallest2(two: Two<K, V>) -> Option<(Item<K, V>, NodeBox<K, V>, bool)> {
        let Two {
            size: _,
            item,
            lhs,
            rhs,
        } = two;
        if lhs.is_empty() {
            Some((item, rhs, true))
        } else {
            let (small, lhs, reduced) = Node::remove_smallest(*lhs).unwrap();
            match reduced {
                true => {
                    let (node, _, reduced) = Node::fix2_lhs(item, lhs, rhs, None);
                    Some((small, node, reduced))
                }
                false => Some((small, Node::two(item, lhs, rhs), false)),
            }
        }
    }

    fn remove_smallest3(three: Three<K, V>) -> Option<(Item<K, V>, NodeBox<K, V>, bool)> {
        let Three {
            size: _,
            item1,
            item2,
            lhs,
            mid,
            rhs,
        } = three;
        if lhs.is_empty() {
            Some((item1, Node::two(item2, mid, rhs), false))
        } else {
            let (small, lhs, reduced) = Node::remove_smallest(*lhs).unwrap();
            match reduced {
                true => {
                    let (node, _, reduced) = Node::fix3_lhs(item1, item2, lhs, mid, rhs, None);
                    Some((small, node, reduced))
                }
                false => Some((small, Node::three(item1, item2, lhs, mid, rhs), false)),
            }
        }
    }

    fn remove_smallest4(four: Four<K, V>) -> Option<(Item<K, V>, NodeBox<K, V>, bool)> {
        let Four {
            size: _,
            item1,
            item2,
            item3,
            lhs,
            lhs_mid,
            rhs_mid,
            rhs,
        } = four;
        if lhs.is_empty() {
            Some((
                item1,
                Node::three(item2, item3, lhs_mid, rhs_mid, rhs),
                false,
            ))
        } else {
            let (small, lhs, reduced) = Node::remove_smallest(*lhs).unwrap();
            match reduced {
                true => {
                    let (node, _, reduced) =
                        Node::fix4_lhs(item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs, None);
                    Some((small, node, reduced))
                }
                false => Some((
                    small,
                    Node::four(item1, item2, item3, lhs, lhs_mid, rhs_mid, rhs),
                    false,
                )),
            }
        }
    }

    pub fn visit<Visitor: FnMut(&Item<K, V>)>(&self, visitor: &mut Visitor) {
        match self {
            Node::Empty => {}
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                lhs.visit(visitor);
                visitor(item);
                rhs.visit(visitor);
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                lhs.visit(visitor);
                visitor(item1);
                mid.visit(visitor);
                visitor(item2);
                rhs.visit(visitor);
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                lhs.visit(visitor);
                visitor(item1);
                lhs_mid.visit(visitor);
                visitor(item2);
                rhs_mid.visit(visitor);
                visitor(item3);
                rhs.visit(visitor);
            }
        }
    }

    fn fix2_lhs(
        orig_item: Item<K, V>,
        orig_lhs: NodeBox<K, V>,
        orig_rhs: NodeBox<K, V>,
        result: Option<V>,
    ) -> (NodeBox<K, V>, Option<V>, bool) {
        match *orig_rhs {
            Node::Empty => unreachable!(),
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                (
                    Node::three(orig_item, item, orig_lhs, lhs, rhs),
                    result,
                    true,
                )
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                let node1 = Node::two(orig_item, orig_lhs, lhs);
                let node2 = Node::two(item2, mid, rhs);
                (Node::two(item1, node1, node2), result, false)
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                let node1 = Node::three(orig_item, item1, orig_lhs, lhs, lhs_mid);
                let node2 = Node::two(item3, rhs_mid, rhs);
                (Node::two(item2, node1, node2), result, false)
            }
        }
    }

    fn fix2_rhs(
        orig_item: Item<K, V>,
        orig_lhs: NodeBox<K, V>,
        orig_rhs: NodeBox<K, V>,
        result: Option<V>,
    ) -> (NodeBox<K, V>, Option<V>, bool) {
        match *orig_lhs {
            Node::Empty => unreachable!(),
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                (
                    Node::three(item, orig_item, lhs, rhs, orig_rhs),
                    result,
                    true,
                )
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                let node1 = Node::two(item1, lhs, mid);
                let node2 = Node::two(orig_item, rhs, orig_rhs);
                (Node::two(item2, node1, node2), result, false)
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                let node1 = Node::three(item1, item2, lhs, lhs_mid, rhs_mid);
                let node2 = Node::two(orig_item, rhs, orig_rhs);
                (Node::two(item3, node1, node2), result, false)
            }
        }
    }

    fn fix3_lhs(
        orig_item1: Item<K, V>,
        orig_item2: Item<K, V>,
        orig_lhs: NodeBox<K, V>,
        orig_mid: NodeBox<K, V>,
        orig_rhs: NodeBox<K, V>,
        result: Option<V>,
    ) -> (NodeBox<K, V>, Option<V>, bool) {
        match *orig_mid {
            Node::Empty => unreachable!(),
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                let node = Node::three(orig_item1, item, orig_lhs, lhs, rhs);
                (Node::two(orig_item2, node, orig_rhs), result, false)
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                let node1 = Node::two(orig_item1, orig_lhs, lhs);
                let node2 = Node::two(item2, mid, rhs);
                (
                    Node::three(item1, orig_item2, node1, node2, orig_rhs),
                    result,
                    false,
                )
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                let node1 = Node::two(orig_item1, orig_lhs, lhs);
                let node2 = Node::three(item2, item3, lhs_mid, rhs_mid, rhs);
                (
                    Node::three(item1, orig_item2, node1, node2, orig_rhs),
                    result,
                    false,
                )
            }
        }
    }

    fn fix3_mid(
        orig_item1: Item<K, V>,
        orig_item2: Item<K, V>,
        orig_lhs: NodeBox<K, V>,
        orig_mid: NodeBox<K, V>,
        orig_rhs: NodeBox<K, V>,
        result: Option<V>,
    ) -> (NodeBox<K, V>, Option<V>, bool) {
        match *orig_lhs {
            Node::Empty => unreachable!(),
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                let node = Node::three(item, orig_item1, lhs, rhs, orig_mid);
                (Node::two(orig_item2, node, orig_rhs), result, false)
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                let node1 = Node::two(item1, lhs, mid);
                let node2 = Node::two(orig_item1, rhs, orig_mid);
                (
                    Node::three(item2, orig_item2, node1, node2, orig_rhs),
                    result,
                    false,
                )
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                let node1 = Node::three(item1, item2, lhs, lhs_mid, rhs_mid);
                let node2 = Node::two(orig_item1, rhs, orig_mid);
                (Node::two(item3, node1, node2), result, false)
            }
        }
    }

    fn fix3_rhs(
        orig_item1: Item<K, V>,
        orig_item2: Item<K, V>,
        orig_lhs: NodeBox<K, V>,
        orig_mid: NodeBox<K, V>,
        orig_rhs: NodeBox<K, V>,
        result: Option<V>,
    ) -> (NodeBox<K, V>, Option<V>, bool) {
        match *orig_mid {
            Node::Empty => unreachable!(),
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                let node = Node::three(item, orig_item2, lhs, rhs, orig_rhs);
                (Node::two(orig_item1, orig_lhs, node), result, false)
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                let node1 = Node::two(item1, lhs, mid);
                let node2 = Node::two(orig_item2, rhs, orig_rhs);
                (
                    Node::three(orig_item1, item2, orig_lhs, node1, node2),
                    result,
                    false,
                )
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                let node1 = Node::three(item1, item2, lhs, lhs_mid, rhs_mid);
                let node2 = Node::two(orig_item2, rhs, orig_rhs);
                (
                    Node::three(orig_item1, item3, orig_lhs, node1, node2),
                    result,
                    false,
                )
            }
        }
    }

    fn fix4_lhs(
        orig_item1: Item<K, V>,
        orig_item2: Item<K, V>,
        orig_item3: Item<K, V>,
        orig_lhs: NodeBox<K, V>,
        orig_lhs_mid: NodeBox<K, V>,
        orig_rhs_mid: NodeBox<K, V>,
        orig_rhs: NodeBox<K, V>,
        result: Option<V>,
    ) -> (NodeBox<K, V>, Option<V>, bool) {
        match *orig_lhs_mid {
            Node::Empty => unreachable!(),
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                let node = Node::three(orig_item1, item, orig_lhs, lhs, rhs);
                (
                    Node::three(orig_item2, orig_item3, node, orig_rhs_mid, orig_rhs),
                    result,
                    false,
                )
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                let node1 = Node::two(orig_item1, orig_lhs, lhs);
                let node2 = Node::two(item2, mid, rhs);
                (
                    Node::four(
                        item1,
                        orig_item2,
                        orig_item3,
                        node1,
                        node2,
                        orig_rhs_mid,
                        orig_rhs,
                    ),
                    result,
                    false,
                )
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                let node1 = Node::two(orig_item1, orig_lhs, lhs);
                let node2 = Node::three(item2, item3, lhs_mid, rhs_mid, rhs);
                (
                    Node::four(
                        item1,
                        orig_item2,
                        orig_item3,
                        node1,
                        node2,
                        orig_rhs_mid,
                        orig_rhs,
                    ),
                    result,
                    false,
                )
            }
        }
    }

    fn fix4_lhs_mid(
        orig_item1: Item<K, V>,
        orig_item2: Item<K, V>,
        orig_item3: Item<K, V>,
        orig_lhs: NodeBox<K, V>,
        orig_lhs_mid: NodeBox<K, V>,
        orig_rhs_mid: NodeBox<K, V>,
        orig_rhs: NodeBox<K, V>,
        result: Option<V>,
    ) -> (NodeBox<K, V>, Option<V>, bool) {
        match *orig_rhs_mid {
            Node::Empty => unreachable!(),
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                let node = Node::three(orig_item2, item, orig_lhs_mid, lhs, rhs);
                (
                    Node::three(orig_item1, orig_item3, orig_lhs, node, orig_rhs),
                    result,
                    false,
                )
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                let node1 = Node::two(orig_item2, orig_lhs_mid, lhs);
                let node2 = Node::two(item2, mid, rhs);
                (
                    Node::four(
                        orig_item1, item1, orig_item3, orig_lhs, node1, node2, orig_rhs,
                    ),
                    result,
                    false,
                )
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                let node1 = Node::two(orig_item2, orig_lhs_mid, lhs);
                let node2 = Node::three(item2, item3, lhs_mid, rhs_mid, rhs);
                (
                    Node::four(
                        orig_item1, item1, orig_item3, orig_lhs, node1, node2, orig_rhs,
                    ),
                    result,
                    false,
                )
            }
        }
    }

    fn fix4_rhs_mid(
        orig_item1: Item<K, V>,
        orig_item2: Item<K, V>,
        orig_item3: Item<K, V>,
        orig_lhs: NodeBox<K, V>,
        orig_lhs_mid: NodeBox<K, V>,
        orig_rhs_mid: NodeBox<K, V>,
        orig_rhs: NodeBox<K, V>,
        result: Option<V>,
    ) -> (NodeBox<K, V>, Option<V>, bool) {
        match *orig_rhs {
            Node::Empty => unreachable!(),
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                let node = Node::three(orig_item3, item, orig_rhs_mid, lhs, rhs);
                (
                    Node::three(orig_item1, orig_item2, orig_lhs, orig_lhs_mid, node),
                    result,
                    false,
                )
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                let node1 = Node::two(orig_item3, orig_rhs_mid, lhs);
                let node2 = Node::two(item2, mid, rhs);
                (
                    Node::four(
                        orig_item1,
                        orig_item2,
                        item1,
                        orig_lhs,
                        orig_lhs_mid,
                        node1,
                        node2,
                    ),
                    result,
                    false,
                )
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                let node1 = Node::two(orig_item3, orig_rhs_mid, lhs);
                let node2 = Node::three(item2, item3, lhs_mid, rhs_mid, rhs);
                (
                    Node::four(
                        orig_item1,
                        orig_item2,
                        item1,
                        orig_lhs,
                        orig_lhs_mid,
                        node1,
                        node2,
                    ),
                    result,
                    false,
                )
            }
        }
    }

    fn fix4_rhs(
        orig_item1: Item<K, V>,
        orig_item2: Item<K, V>,
        orig_item3: Item<K, V>,
        orig_lhs: NodeBox<K, V>,
        orig_lhs_mid: NodeBox<K, V>,
        orig_rhs_mid: NodeBox<K, V>,
        orig_rhs: NodeBox<K, V>,
        result: Option<V>,
    ) -> (NodeBox<K, V>, Option<V>, bool) {
        match *orig_rhs_mid {
            Node::Empty => unreachable!(),
            Node::Two(two) => {
                let Two {
                    size: _,
                    item,
                    lhs,
                    rhs,
                } = two;
                let node = Node::three(item, orig_item3, lhs, rhs, orig_rhs);
                (
                    Node::three(orig_item1, orig_item2, orig_lhs, orig_lhs_mid, node),
                    result,
                    false,
                )
            }
            Node::Three(three) => {
                let Three {
                    size: _,
                    item1,
                    item2,
                    lhs,
                    mid,
                    rhs,
                } = three;
                let node1 = Node::two(item1, lhs, mid);
                let node2 = Node::two(orig_item3, rhs, orig_rhs);
                (
                    Node::four(
                        orig_item1,
                        orig_item2,
                        item2,
                        orig_lhs,
                        orig_lhs_mid,
                        node1,
                        node2,
                    ),
                    result,
                    false,
                )
            }
            Node::Four(four) => {
                let Four {
                    size: _,
                    item1,
                    item2,
                    item3,
                    lhs,
                    lhs_mid,
                    rhs_mid,
                    rhs,
                } = four;
                let node1 = Node::three(item1, item2, lhs, lhs_mid, rhs_mid);
                let node2 = Node::two(orig_item3, rhs, orig_rhs);
                (
                    Node::four(
                        orig_item1,
                        orig_item2,
                        item3,
                        orig_lhs,
                        orig_lhs_mid,
                        node1,
                        node2,
                    ),
                    result,
                    false,
                )
            }
        }
    }
}

type NodeBox<K, V> = Box<Node<K, V>>;

struct Two<K: Eq + Ord, V> {
    size: usize,
    item: Item<K, V>,
    lhs: NodeBox<K, V>,
    rhs: NodeBox<K, V>,
}

struct Three<K: Eq + Ord, V> {
    size: usize,
    item1: Item<K, V>,
    item2: Item<K, V>,
    lhs: NodeBox<K, V>,
    mid: NodeBox<K, V>,
    rhs: NodeBox<K, V>,
}

struct Four<K: Eq + Ord, V> {
    size: usize,
    item1: Item<K, V>,
    item2: Item<K, V>,
    item3: Item<K, V>,
    lhs: NodeBox<K, V>,
    lhs_mid: NodeBox<K, V>,
    rhs_mid: NodeBox<K, V>,
    rhs: NodeBox<K, V>,
}

pub struct Tree234<K: Eq + Ord, V> {
    root: NodeBox<K, V>,
}

impl<K: Eq + Ord, V> Tree234<K, V> {
    pub fn new() -> Tree234<K, V> {
        Tree234 {
            root: Node::empty(),
        }
    }

    pub fn size(&self) -> usize {
        self.root.size()
    }

    pub fn get(&self, key: &K) -> Option<&Item<K, V>> {
        self.root.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut root = Node::empty();
        std::mem::swap(&mut self.root, &mut root);
        let root = root.insert(key, value);
        self.root = root;
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut root = Node::empty();
        std::mem::swap(&mut self.root, &mut root);
        let (root, result, _reduced) = root.remove(key);
        self.root = root;
        result
    }

    pub fn clear(&mut self) {
        self.root = Node::empty();
    }

    pub fn visit<Visitor: FnMut(&Item<K, V>)>(&self, visitor: &mut Visitor) {
        self.root.visit(visitor);
    }

    pub fn iter(&self) -> Tree234Iterator<'_, K, V> {
        Tree234Iterator::new(self)
    }
}

pub struct Tree234Iterator<'a, K: Eq + Ord, V> {
    items: VecDeque<Either<&'a Item<K, V>, &'a NodeBox<K, V>>>,
}

impl<'a, K: Eq + Ord, V> Tree234Iterator<'a, K, V> {
    pub fn new(tree: &'a Tree234<K, V>) -> Tree234Iterator<'a, K, V> {
        let mut items = VecDeque::new();
        items.push_back(Either::Right(&tree.root));
        Tree234Iterator { items: items }
    }
}

impl<'a, K: Eq + Ord, V> Iterator for Tree234Iterator<'a, K, V> {
    type Item = &'a (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(thing) = self.items.pop_front() {
            match thing {
                Either::Left(item) => return Some(item),
                Either::Right(node) => {
                    match node.as_ref() {
                        Node::Empty => {
                            // do nothing!
                        }
                        Node::Two(two) => {
                            let Two {
                                size: _,
                                item,
                                lhs,
                                rhs,
                            } = two;
                            self.items.push_front(Either::Right(rhs));
                            self.items.push_front(Either::Left(item));
                            self.items.push_front(Either::Right(lhs));
                        }
                        Node::Three(three) => {
                            let Three {
                                size: _,
                                item1,
                                item2,
                                lhs,
                                mid,
                                rhs,
                            } = three;
                            self.items.push_front(Either::Right(rhs));
                            self.items.push_front(Either::Left(item2));
                            self.items.push_front(Either::Right(mid));
                            self.items.push_front(Either::Left(item1));
                            self.items.push_front(Either::Right(lhs));
                        }
                        Node::Four(four) => {
                            let Four {
                                size: _,
                                item1,
                                item2,
                                item3,
                                lhs,
                                lhs_mid,
                                rhs_mid,
                                rhs,
                            } = four;
                            self.items.push_front(Either::Right(rhs));
                            self.items.push_front(Either::Left(item3));
                            self.items.push_front(Either::Right(rhs_mid));
                            self.items.push_front(Either::Left(item2));
                            self.items.push_front(Either::Right(lhs_mid));
                            self.items.push_front(Either::Left(item1));
                            self.items.push_front(Either::Right(lhs));
                        }
                    }
                }
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::seq::SliceRandom;
    use rand::{Rng, SeedableRng};

    use super::*;

    #[test]
    fn empty_1() {
        let tree: Tree234<i32, usize> = Tree234::new();
        assert_eq!(tree.size(), 0);
        assert_eq!(tree.get(&1), None);
    }

    #[test]
    fn insert_1() {
        let mut tree: Tree234<i32, f64> = Tree234::new();
        assert_eq!(tree.size(), 0);
        tree.insert(5, 3.0);
        assert_eq!(tree.size(), 1);
        tree.insert(4, 3.1);
        assert_eq!(tree.size(), 2);
    }

    #[test]
    fn insert_2() {
        let xs: Vec<i32> = vec![
            0, 22, 11, 2, 16, 4, 18, 7, 23, 15, 14, 5, 10, 17, 19, 20, 6, 24, 8, 12, 13, 9, 3, 21,
            1,
        ];
        let n = xs.len();
        let mut tree: Tree234<i32, usize> = Tree234::new();
        assert_eq!(tree.size(), 0);
        for i in 0..xs.len() {
            let x = xs[i];
            tree.insert(x, i);
            assert_eq!(tree.size(), i + 1);
        }
        for i in 0..xs.len() {
            let x = xs[i];
            let r = tree.get(&x);
            assert_eq!(r, Some(&(x, i)));
        }
        for i in 0..xs.len() {
            assert_eq!(tree.size(), n - i);
            let x = xs[i];
            let r = tree.remove(&x);
            assert_eq!(r, Some(i));
        }
    }

    #[test]
    fn visitor_1() {
        let n: u64 = 1000;
        let mut tree: Tree234<u64, u64> = Tree234::new();
        for i in 0..n {
            let x = 0x3fff_u64
                .wrapping_add(i)
                .wrapping_mul(0x9e3779b97f4a7c13u64)
                & 0xffffffffu64;
            tree.insert(x, i);
        }
        let mut xs: Vec<u64> = Vec::new();
        tree.visit(&mut |item| xs.push(item.0));
        assert_eq!(xs.len(), n as usize);
        for i in 1..(n as usize) {
            assert!(xs[i - 1] < xs[i]);
        }

        // permute xs
        for i in 0..(n as usize) {
            let j = 0x3ff_usize.wrapping_add(i).wrapping_mul(0x9e3779b9usize) % (n as usize);
            let t = xs[i];
            xs[i] = xs[j];
            xs[j] = t;
        }

        for i in 0..(n as usize) {
            let x = xs[i];
            let r = tree.remove(&x);
            assert!(r.is_some());
            let j = r.unwrap();
            let y = 0x3fff_u64
                .wrapping_add(j)
                .wrapping_mul(0x9e3779b97f4a7c13u64)
                & 0xffffffffu64;
            assert_eq!(x, y);
        }
    }

    #[test]
    fn iterator_1() {
        let xs: Vec<i32> = vec![
            0, 22, 11, 2, 16, 4, 18, 7, 23, 15, 14, 5, 10, 17, 19, 20, 6, 24, 8, 12, 13, 9, 3, 21,
            1,
        ];
        let n = xs.len();
        let mut tree: Tree234<i32, usize> = Tree234::new();
        assert_eq!(tree.size(), 0);
        for i in 0..xs.len() {
            let x = xs[i];
            tree.insert(x, i);
            assert_eq!(tree.size(), i + 1);
        }
        let ys = tree.iter().map(|item| item.clone()).collect::<Vec<(i32,usize)>>();
        assert_eq!(xs.len(), ys.len());
        for i in 0..n {
            assert_eq!(ys[i].0, i as i32);
        }
    }
    
    #[test]
    fn structures_1() {
        let mut rng = StdRng::seed_from_u64(17u64);
        let n: usize = 100;
        for _i in 0..200 {
            let mut tree: Tree234<u64, u64> = Tree234::new();
            let mut xs: Vec<u64> = Vec::new();
            for j in 0..n {
                let x: u64 = rng.gen::<u64>() & 0xffffff;
                xs.push(x);
                tree.insert(x, j as u64);
                let y: u64 = rng.gen::<u64>() & 0xffffff;
                tree.insert(y, j as u64);
            }
            xs.shuffle(&mut rng);
            for x in xs.iter() {
                tree.remove(x);
            }
        }
    }

    #[test]
    #[ignore]
    fn exhaustion_1() {
        let mut rng = StdRng::seed_from_u64(17u64);
        let mut tree: Tree234<u64, u64> = Tree234::new();
        let n: u64 = 0x1_000_000;
        let mut rate = 0.5;
        for i in 0..n {
            if rng.gen::<f64>() < 1e-5 {
                tree.clear();
            }
            if i & 0xFFFF == 0 {
                rate = rng.gen::<f64>();
            }
            let x = rng.gen::<u64>() & 0xFFFF;
            if rng.gen::<f64>() < rate {
                tree.insert(x, i);
            } else {
                tree.remove(&x);
            }
            if rng.gen::<f64>() < 0.0001 {
                //eprintln!("[{}]\ttree.size() = {}", i, tree.size());
                let mut xs: Vec<u64> = Vec::new();
                tree.visit(&mut |item| xs.push(item.0));
                assert_eq!(tree.size(), xs.len());
                for j in 1..xs.len() {
                    assert!(xs[j - 1] < xs[j]);
                }
            }
        }
    }
}
