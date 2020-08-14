use std::sync::atomic::{AtomicUsize, Ordering};

use rand::{Rng, thread_rng};

use crate::config::{self, RequestOrder};

#[cfg(test)]
mod tests;

pub(crate) fn url_source(config: &config::Config) -> impl Iterator<Item = &str> {
    // Create the requested order generator
    let limit = config.urls.len();
    let requests = config.requests;
    let index_iter: Box<dyn Iterator<Item=usize>> = match config.order {
        RequestOrder::Sequential => Box::new(SequentialIter::new(limit, requests)),
        RequestOrder::Random => Box::new(RandomIter::new(limit, requests)),
    };

    UrlSource { urls: &config.urls, index_iter: Box::new(index_iter) }
}

struct UrlSource<'a> {
    urls: &'a Vec<String>,
    index_iter: Box<dyn Iterator<Item=usize>>,
}

impl<'a> Iterator for UrlSource<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.index_iter.next()
            .map(|i| self.urls[i].as_str())
    }
}

struct RandomIter {
    count: AtomicUsize,
    limit: usize,
    requests: usize,
}

impl RandomIter {
    pub fn new(limit: usize, requests: usize) -> RandomIter {
        RandomIter { count: AtomicUsize::new(0), limit, requests }
    }
}

impl Iterator for RandomIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // Bump the number of URLs we have generated
        let index = self.count.fetch_add(1, Ordering::Relaxed);

        // Extract the next URL if we haven't generated sufficient URLs
        if index < self.requests {
            Some(thread_rng().gen_range(0, self.limit))
        } else {
            None
        }
    }
}


pub struct SequentialIter {
    next: AtomicUsize,
    limit: usize,
    requests: usize,
}

impl SequentialIter {
    pub fn new(limit: usize, requests: usize) -> SequentialIter {
        SequentialIter { next: AtomicUsize::new(0), limit, requests }
    }
}

impl Iterator for SequentialIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // Fetch the next index in the url list
        let index = self.next.fetch_add(1, Ordering::Relaxed);

        // Extract the next URL if we haven't generated sufficient URLs
        if index < self.requests {
            Some(index % self.limit)
        } else {
            None
        }
    }
}
