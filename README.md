# Github webhook

参考了<https://github.com/Yurunsoft/rust-webhook>，在此基础上写成

将配置文件改为了yaml，但未添加gitee的支持

将框架改为了axum

将执行输出改为了日志输出，方便统一输出

增加了日志文件的输出，方便追踪日志

将配置文件的部分可选项改为了Option，不必为了不panic而留个空参数在那