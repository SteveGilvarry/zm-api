import { Test, TestingModule } from '@nestjs/testing';
import { StatesResolver } from './states.resolver';
import { StatesService } from './states.service';

describe('StatesResolver', () => {
  let resolver: StatesResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [StatesResolver, StatesService],
    }).compile();

    resolver = module.get<StatesResolver>(StatesResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
