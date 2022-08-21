import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersCreateInput } from './servers-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneServersArgs {

    @Field(() => ServersCreateInput, {nullable:false})
    @Type(() => ServersCreateInput)
    data!: ServersCreateInput;
}
