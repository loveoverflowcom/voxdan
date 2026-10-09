# Deferred expansion

These ideas are outside the first production-to-publication and listening workflow. Promote an item only when real usage, implementation evidence, or an explicit product decision makes it necessary. Give it a separate review boundary rather than expanding an active MVP item.

| Idea | Why deferred | Evidence needed to promote |
| --- | --- | --- |
| Independent microservices, event buses, and distributed orchestration | A modular backend with durable PostgreSQL jobs serves the current product flow. | Measured ownership, deployment, scaling, or reliability limits that a smaller change cannot address |
| Provider auto-routing, large catalogs, generation batching, and local model tuning | Start with usable provider paths and correct recovery/cost behavior. | Quality, latency, cost, or hardware data from actual production samples |
| Real-time collaborative Studio, full DAW controls, and mobile Studio | Single-creator editing and focused production controls deliver the first workflow. | Demonstrated collaborative or editing tasks blocked by the MVP |
| Direct Narrative Forge integration and additional import formats | Versioned Script IR keeps integration possible without coupling projects now. | A concrete source/consumer, fixtures, and compatibility ownership |
| Personalized recommendations, subscriptions, payments, and social features | Discovery and reliable listening must work first. | Defined business rules and audience needs after usable published content exists |
| Automated perceptual QC, music generation, and advanced mastering | Measurable QC plus creator listening/approval provides the initial gate. | Evaluation material and a verified quality improvement over the current review process |
| Multi-region delivery, CDN tuning, archival automation, and aggressive deduplication | Correct delivery, version retention, and safe uploads precede optimization. | Storage/traffic/cost measurements and a retention policy that protects active playback and offline clients |
| Wearables, car interfaces, live broadcasting, and DRM | They have no immediate MVP consumer. | An explicit platform/distribution requirement and native runtime validation capacity |

Data loss, permission bypass, unbounded billable retries, invalid publication, unusable media, and corrupt offline state are correctness fixes—not deferred optimization.
