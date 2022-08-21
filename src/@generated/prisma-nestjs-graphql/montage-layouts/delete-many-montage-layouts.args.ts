import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsWhereInput } from './montage-layouts-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyMontageLayoutsArgs {

    @Field(() => MontageLayoutsWhereInput, {nullable:true})
    @Type(() => MontageLayoutsWhereInput)
    where?: MontageLayoutsWhereInput;
}
