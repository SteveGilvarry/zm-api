import { Test, TestingModule } from '@nestjs/testing';
import { MonitorpresetsResolver } from './monitorpresets.resolver';
import { MonitorpresetsService } from './monitorpresets.service';

describe('MonitorpresetsResolver', () => {
  let resolver: MonitorpresetsResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MonitorpresetsResolver, MonitorpresetsService],
    }).compile();

    resolver = module.get<MonitorpresetsResolver>(MonitorpresetsResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
