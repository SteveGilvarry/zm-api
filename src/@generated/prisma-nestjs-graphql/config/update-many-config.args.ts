import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigUpdateManyMutationInput } from './config-update-many-mutation.input';
import { Type } from 'class-transformer';
import { ConfigWhereInput } from './config-where.input';

@ArgsType()
export class UpdateManyConfigArgs {

    @Field(() => ConfigUpdateManyMutationInput, {nullable:false})
    @Type(() => ConfigUpdateManyMutationInput)
    data!: ConfigUpdateManyMutationInput;

    @Field(() => ConfigWhereInput, {nullable:true})
    @Type(() => ConfigWhereInput)
    where?: ConfigWhereInput;
}
