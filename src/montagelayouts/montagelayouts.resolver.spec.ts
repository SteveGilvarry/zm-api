import { Test, TestingModule } from '@nestjs/testing';
import { MontagelayoutsResolver } from './montagelayouts.resolver';
import { MontagelayoutsService } from './montagelayouts.service';

describe('MontagelayoutsResolver', () => {
  let resolver: MontagelayoutsResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [MontagelayoutsResolver, MontagelayoutsService],
    }).compile();

    resolver = module.get<MontagelayoutsResolver>(MontagelayoutsResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
