import { Test, TestingModule } from '@nestjs/testing';
import { MonitorpresetsService } from './monitorpresets.service';

describe('MonitorpresetsService', () => {
  let service: MonitorpresetsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MonitorpresetsService],
    }).compile();

    service = module.get<MonitorpresetsService>(MonitorpresetsService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
