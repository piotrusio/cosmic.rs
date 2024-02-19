#![allow(dead_code)]
use chrono::{DateTime, Local};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch<'a> {
    pub id: Option<u32>,
    pub sku: String,
    pub qty: u32,
    pub eta: DateTime<Local>,
    pub allocated: Vec<&'a OrderLine>,
}

impl<'a> Batch<'a> {
    pub fn new(sku: String, qty: u32) -> Batch<'a> {
        Batch {
            id: None,
            sku,
            qty,
            eta: Local::now(),
            allocated: Vec::new(),
        }
    }

    pub fn avaialble_qty(&self) -> u32 {
        let allocated_qty: u32 =
            self.allocated.iter().map(|line| line.qty).sum();
        self.qty - allocated_qty
    }

    pub fn allocate(
        &mut self,
        order_line: &'a OrderLine,
    ) -> Result<(), &'static str> {
        if self.sku != order_line.sku {
            return Err("SKU do not match");
        }
        if self.allocated.iter().any(|&x| x == order_line) {
            return Err("Order line already allocated in this batch");
        }
        if self.avaialble_qty() >= order_line.qty {
            self.allocated.push(order_line);
            Ok(())
        } else {
            Err("Not enough quantity in batch to allocate order line")
        }
    }

    pub fn deallocate(
        &mut self,
        order_line: &'a OrderLine,
    ) -> Result<(), &'static str> {
        let position = self.allocated.iter().position(|&x| x == order_line);
        match position {
            Some(index) => {
                self.allocated.remove(index);
                Ok(())
            }
            None => Err("Cannot deallocate unallocated order"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderLine {
    pub id: Option<u32>,
    pub sku: String,
    pub qty: u32,
}

impl OrderLine {
    pub fn new(sku: String, qty: u32) -> OrderLine {
        OrderLine { id: None, sku, qty }
    }
}

pub fn allocate<'a>(
    order_line: &'a OrderLine,
    batches: &mut Vec<&mut Batch<'a>>,
) -> Result<(), &'static str> {
    // Sort batches by eta
    batches.sort_by(|a, b| a.eta.cmp(&b.eta));

    // Try to allocate the order line to each batch
    if batches
        .iter_mut()
        .any(|batch| batch.allocate(order_line).is_ok())
    {
        Ok(()) // If allocation is successful, return Ok(())
    } else {
        // If none of the batches can accommodate the order line, return an error
        Err("Cannot allocate order line to any batch")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_allocating_to_a_batch_reduces_the_available_quantity() {
        let sku = "SMALL_TABLE".to_string();

        let mut batch = Batch::new(sku.clone(), 20);
        let order = OrderLine::new(sku, 2);

        assert_eq!(batch.allocate(&order), Ok(()));
        assert_eq!(batch.avaialble_qty(), 18);
    }

    #[test]
    fn test_allocating_to_a_batch_with_insufficient_quantity() {
        let sku = "SMALL_TABLE".to_string();

        let mut batch = Batch::new(sku.clone(), 1);
        let order = OrderLine::new(sku, 2);

        assert_eq!(
            batch.allocate(&order),
            Err("Not enough quantity in batch to allocate order line")
        );
        assert_eq!(batch.avaialble_qty(), 1);
    }

    #[test]
    fn test_allocating_to_a_batch_with_different_sku() {
        let batch_sku = "SMALL_TABLE".to_string();
        let order_sku = "BIG_TABLE".to_string();

        let mut batch = Batch::new(batch_sku, 10);
        let order = OrderLine::new(order_sku, 2);

        assert_eq!(batch.allocate(&order), Err("SKU do not match"));
        assert_eq!(batch.avaialble_qty(), 10);
    }

    #[test]
    fn test_can_only_deallocate_allocated_lines() {
        let sku = "SMALL_TABLE".to_string();

        let mut batch = Batch::new(sku.clone(), 10);
        let order1 = OrderLine::new(sku.clone(), 3);
        let order2 = OrderLine::new(sku.clone(), 2);

        assert_eq!(batch.allocate(&order1), Ok(()));
        assert_eq!(batch.deallocate(&order1), Ok(()));
        assert_eq!(batch.avaialble_qty(), 10);

        assert_eq!(
            batch.deallocate(&order2),
            Err("Cannot deallocate unallocated order")
        );
        assert_eq!(batch.avaialble_qty(), 10)
    }

    #[test]
    fn test_allocation_is_idempotent() {
        let sku = "SMALL_TABLE".to_string();

        let mut batch = Batch::new(sku.clone(), 20);
        let order = OrderLine::new(sku, 2);

        assert_eq!(batch.allocate(&order), Ok(()));
        assert_eq!(
            batch.allocate(&order),
            Err("Order line already allocated in this batch")
        );
        assert_eq!(batch.avaialble_qty(), 18);
    }

    #[test]
    fn test_prefers_earlier_batches() {
        let sku = "SMALL_TABLE".to_string();

        let mut stock_batch = Batch::new(sku.clone(), 20);
        let mut ship_batch = Batch::new(sku.clone(), 20);
        let mut batches = Vec::new();
        batches.push(&mut ship_batch);
        batches.push(&mut stock_batch);

        let order = OrderLine::new(sku, 10);

        assert_eq!(allocate(&order, &mut batches), Ok(()));
        assert_eq!(stock_batch.avaialble_qty(), 10);
        assert_eq!(ship_batch.avaialble_qty(), 20);
    }
}
