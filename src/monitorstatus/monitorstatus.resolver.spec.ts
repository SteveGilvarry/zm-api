import { Test, TestingModule } from '@nestjs/testing';
import { MonitorstatusResolver } from './monitorstatus.resolver';
import { MonitorstatusService } from './monitorstatus.service';

describe('MonitorstatusResolver', () => {
  let resolver: MonitorstatusResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MonitorstatusResolver, MonitorstatusService],
    }).compile();

    resolver = module.get<MonitorstatusResolver>(MonitorstatusResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
