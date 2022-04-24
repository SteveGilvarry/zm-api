import { Test, TestingModule } from '@nestjs/testing';
import { ServersResolver } from './servers.resolver';
import { ServersService } from './servers.service';

describe('ServersResolver', () => {
  let resolver: ServersResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ServersResolver, ServersService],
    }).compile();

    resolver = module.get<ServersResolver>(ServersResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
