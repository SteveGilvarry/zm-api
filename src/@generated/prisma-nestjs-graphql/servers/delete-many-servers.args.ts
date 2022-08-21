import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersWhereInput } from './servers-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyServersArgs {

    @Field(() => ServersWhereInput, {nullable:true})
    @Type(() => ServersWhereInput)
    where?: ServersWhereInput;
}
