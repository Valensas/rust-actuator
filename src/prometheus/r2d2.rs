use prometheus::{
    core::{Collector, Desc},
    Gauge,
};
use r2d2::{ManageConnection, Pool};

pub struct PoolMetricCollectorConfig {
    min_idle: prometheus::Opts,
    max_pool_size: prometheus::Opts,
    total_connections: prometheus::Opts,
    idle_connections: prometheus::Opts,
}

#[derive(Clone)]
pub struct PoolMetricCollector<T: r2d2::ManageConnection> {
    pool: r2d2::Pool<T>,
    min_idle: Gauge,
    max_pool_size: Gauge,
    total_connections: Gauge,
    idle_connections: Gauge,
    descs: Vec<Desc>,
}

impl PoolMetricCollectorConfig {
    pub fn default(pool_name: &str) -> Self {
        Self {
            min_idle: prometheus::opts!(
                "r2d2_min_idle",
                "r2d2 minimum number of idle connections",
                prometheus::labels! {"name" => pool_name}
            ),
            max_pool_size: prometheus::opts!(
                "r2d2_max_pool_size",
                "r2d2 maximum pool size",
                prometheus::labels! {"name" => pool_name}
            ),
            total_connections: prometheus::opts!(
                "r2d2_connections_total",
                "number of r2d2 active pool size",
                prometheus::labels! {"name" => pool_name}
            ),
            idle_connections: prometheus::opts!(
                "r2d2_connections_idle",
                "number of r2d2 idle connections",
                prometheus::labels! {"name" => pool_name}
            ),
        }
    }
}

impl<T: ManageConnection> PoolMetricCollector<T> {
    pub fn new(
        pool: Pool<T>,
        config: PoolMetricCollectorConfig,
    ) -> Result<Self, prometheus::Error> {
        let min_idle = Gauge::with_opts(config.min_idle)?;
        let max_pool_size = Gauge::with_opts(config.max_pool_size)?;
        let total_connections = Gauge::with_opts(config.total_connections)?;
        let idle_connections = Gauge::with_opts(config.idle_connections)?;
        let mut descs = vec![];

        descs.extend(min_idle.desc().into_iter().cloned());
        descs.extend(max_pool_size.desc().into_iter().cloned());
        descs.extend(total_connections.desc().into_iter().cloned());
        descs.extend(idle_connections.desc().into_iter().cloned());

        let collector = Self {
            pool,
            min_idle,
            max_pool_size,
            total_connections,
            idle_connections,
            descs,
        };

        Ok(collector)
    }
}

impl<T: ManageConnection> Collector for PoolMetricCollector<T> {
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        self.descs.iter().collect()
    }

    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        let mut mfs = Vec::new();

        self.min_idle
            .set(self.pool.min_idle().unwrap_or_default() as f64);
        mfs.extend(self.min_idle.collect());

        self.max_pool_size.set(self.pool.max_size() as f64);
        mfs.extend(self.max_pool_size.collect());

        self.total_connections
            .set(self.pool.state().connections as f64);
        mfs.extend(self.total_connections.collect());

        self.idle_connections
            .set(self.pool.state().idle_connections as f64);
        mfs.extend(self.idle_connections.collect());

        mfs
    }
}
