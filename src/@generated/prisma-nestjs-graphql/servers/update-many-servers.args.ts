import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersUpdateManyMutationInput } from './servers-update-many-mutation.input';
import { Type } from 'class-transformer';
import { ServersWhereInput } from './servers-where.input';

@ArgsType()
export class UpdateManyServersArgs {

    @Field(() => ServersUpdateManyMutationInput, {nullable:false})
    @Type(() => ServersUpdateManyMutationInput)
    data!: ServersUpdateManyMutationInput;

    @Field(() => ServersWhereInput, {nullable:true})
    @Type(() => ServersWhereInput)
    where?: ServersWhereInput;
}
