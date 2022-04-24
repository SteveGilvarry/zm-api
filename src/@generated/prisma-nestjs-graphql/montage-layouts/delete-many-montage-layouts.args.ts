import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsWhereInput } from './montage-layouts-where.input';

@ArgsType()
export class DeleteManyMontageLayoutsArgs {

    @Field(() => MontageLayoutsWhereInput, {nullable:true})
    where?: MontageLayoutsWhereInput;
}
