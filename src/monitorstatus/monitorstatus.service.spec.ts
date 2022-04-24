import { Test, TestingModule } from '@nestjs/testing';
import { MonitorstatusService } from './monitorstatus.service';

describe('MonitorstatusService', () => {
  let service: MonitorstatusService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MonitorstatusService],
    }).compile();

    service = module.get<MonitorstatusService>(MonitorstatusService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
