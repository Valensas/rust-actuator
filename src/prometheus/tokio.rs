use prometheus::{
    core::{Collector, Desc},
    Gauge,
};

pub struct RuntimeMetricCollectorConfig {
    runtime: tokio::runtime::Handle,
    workers_total: prometheus::Opts,
    blocking_threads_total: prometheus::Opts,
    active_tasks_total: prometheus::Opts,
    idle_blocking_threads_total: prometheus::Opts,
    forced_yield_total: prometheus::Opts,
    injection_queue_size: prometheus::Opts,
    blocking_queue_depth: prometheus::Opts,
}

pub struct RuntimeMetricCollector {
    runtime: tokio::runtime::Handle,
    workers_total: Gauge,
    blocking_threads_total: Gauge,
    active_tasks_total: Gauge,
    idle_blocking_threads_total: Gauge,
    forced_yield_total: Gauge,
    injection_queue_size: Gauge,
    blocking_queue_depth: Gauge,
    descs: Vec<Desc>,
}

impl RuntimeMetricCollectorConfig {
    pub fn default(runtime: tokio::runtime::Handle) -> Self {
        let runtime_id = &runtime.id().to_string();

        Self {
            runtime,
            workers_total: prometheus::opts!(
                "tokio_workers_total",
                "tokio_workers_total",
                prometheus::labels! {"id" => runtime_id}
            ),
            blocking_threads_total: prometheus::opts!(
                "tokio_blocking_threads_total",
                "tokio_blocking_threads_total",
                prometheus::labels! {"id" => runtime_id}
            ),
            active_tasks_total: prometheus::opts!(
                "tokio_active_tasks_total",
                "tokio_active_tasks_total",
                prometheus::labels! {"id" => runtime_id}
            ),
            idle_blocking_threads_total: prometheus::opts!(
                "tokio_idle_blocking_threads_total",
                "tokio_idle_blocking_threads_total",
                prometheus::labels! {"id" => runtime_id}
            ),
            forced_yield_total: prometheus::opts!(
                "tokio_forced_yield_total",
                "tokio_forced_yield_total",
                prometheus::labels! {"id" => runtime_id}
            ),
            injection_queue_size: prometheus::opts!(
                "tokio_injection_queue_size",
                "tokio_injection_queue_size",
                prometheus::labels! {"id" => runtime_id}
            ),
            blocking_queue_depth: prometheus::opts!(
                "tokio_blocking_queue_depth",
                "tokio_blocking_queue_depth",
                prometheus::labels! {"id" => runtime_id}
            ),
        }
    }
}

impl RuntimeMetricCollector {
    pub fn new(config: RuntimeMetricCollectorConfig) -> Result<Self, prometheus::Error> {
        let workers_total = Gauge::with_opts(config.workers_total)?;
        let blocking_threads_total = Gauge::with_opts(config.blocking_threads_total)?;
        let active_tasks_total = Gauge::with_opts(config.active_tasks_total)?;
        let idle_blocking_threads_total = Gauge::with_opts(config.idle_blocking_threads_total)?;
        let forced_yield_total = Gauge::with_opts(config.forced_yield_total)?;
        let injection_queue_size = Gauge::with_opts(config.injection_queue_size)?;
        let blocking_queue_depth = Gauge::with_opts(config.blocking_queue_depth)?;

        let mut descs = vec![];

        descs.extend(workers_total.desc().into_iter().cloned());
        descs.extend(blocking_threads_total.desc().into_iter().cloned());
        descs.extend(active_tasks_total.desc().into_iter().cloned());
        descs.extend(idle_blocking_threads_total.desc().into_iter().cloned());
        descs.extend(forced_yield_total.desc().into_iter().cloned());
        descs.extend(injection_queue_size.desc().into_iter().cloned());
        descs.extend(blocking_queue_depth.desc().into_iter().cloned());

        let collector = Self {
            runtime: config.runtime,
            workers_total,
            blocking_threads_total,
            active_tasks_total,
            idle_blocking_threads_total,
            forced_yield_total,
            injection_queue_size,
            blocking_queue_depth,
            descs,
        };

        Ok(collector)
    }
}

impl Collector for RuntimeMetricCollector {
    fn desc(&self) -> Vec<&Desc> {
        self.descs.iter().collect()
    }

    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        let mut mfs = Vec::new();
        let metrics = self.runtime.metrics();

        self.workers_total.set(metrics.num_workers() as f64);
        mfs.extend(self.workers_total.collect());

        self.blocking_threads_total
            .set(metrics.num_blocking_threads() as f64);
        mfs.extend(self.blocking_threads_total.collect());

        self.active_tasks_total
            .set(metrics.active_tasks_count() as f64);
        mfs.extend(self.active_tasks_total.collect());

        self.idle_blocking_threads_total
            .set(metrics.num_idle_blocking_threads() as f64);
        mfs.extend(self.idle_blocking_threads_total.collect());

        self.forced_yield_total
            .set(metrics.budget_forced_yield_count() as f64);
        mfs.extend(self.forced_yield_total.collect());

        self.injection_queue_size
            .set(metrics.injection_queue_depth() as f64);
        mfs.extend(self.injection_queue_size.collect());

        self.blocking_queue_depth
            .set(metrics.blocking_queue_depth() as f64);
        mfs.extend(self.blocking_queue_depth.collect());

        mfs
    }
}
