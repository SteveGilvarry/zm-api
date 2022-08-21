import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigWhereInput } from './config-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyConfigArgs {

    @Field(() => ConfigWhereInput, {nullable:true})
    @Type(() => ConfigWhereInput)
    where?: ConfigWhereInput;
}
