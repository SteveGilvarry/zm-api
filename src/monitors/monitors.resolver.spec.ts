import { Test, TestingModule } from '@nestjs/testing';
import { MonitorsResolver } from './monitors.resolver';
import { MonitorsService } from './monitors.service';

describe('MonitorsResolver', () => {
  let resolver: MonitorsResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MonitorsResolver, MonitorsService],
    }).compile();

    resolver = module.get<MonitorsResolver>(MonitorsResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
